pub mod tcp_client;

use std::cell::Cell;
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::{Arc, Mutex, RwLock};
use std::sync::mpsc::{Receiver, SyncSender};

use serde::{Deserialize, Serialize};

use crate::service::{ServiceApi, ServiceInitializer, Context};

pub struct RequestAsyncReceiver<I: Send, O: Send> {
    request_rx: Receiver<I>,
    response_tx: SyncSender<O>,
    pending_response: Cell<bool>,
}

impl <I: Send, O: Send> RequestAsyncReceiver<I, O> {

    pub fn try_receive(&self) -> Option<I> {
        let req = self.request_rx.try_recv().ok();
        if req.is_some() {
            self.pending_response.set(true);
        }
        req
    }

    pub fn send_response(&self, response: O) {
        self.response_tx.send(response).unwrap();
        self.pending_response.set(false);
    }

    pub fn is_pending_response(&self) -> bool {
        self.pending_response.get()
    }

}

pub struct AsyncHandler<I: Send, O: Send> {
    request_rx: Receiver<I>,
    response_tx: SyncSender<O>,
    handler: Box<dyn Fn(&I) -> O + 'static>,
}

impl <I: Send, O: Send> AsyncHandler<I, O> {

    pub fn try_handle(&self) {
        match self.request_rx.try_recv() {
            Ok(req) => {
                let handler = self.handler.deref();
                let resp: O = handler(&req);
                self.response_tx.send(resp).unwrap();
            },
            Err(_) => {

            }
        }
    }

}

struct Listener {
    handler: Box<dyn Fn(&str) -> String + Sync + Send + 'static>,
}

struct GetFileListener {
    handler: Box<dyn Fn(&str) -> Result<Vec<u8>, std::io::Error> + Sync + Send + 'static>,
}

#[derive(Serialize, Deserialize)]
pub struct EmptyData {
    pub value: Option<i32>,
}

impl EmptyData {

    pub fn new() -> EmptyData {
        EmptyData {
            value: None,
        }
    }

}

pub struct Rpc {
    calls: RwLock<HashMap<String, Listener>>,
    get_file_calls: RwLock<HashMap<String, GetFileListener>>,
}

impl Rpc {

    pub fn new() -> Self {
        Self {
            calls: RwLock::new(HashMap::new()),
            get_file_calls: RwLock::new(HashMap::new()),
        }
    }

    pub fn on_generic_call_fn<I, O, F>(&self, key: &str, handler: F) where
            for<'de> I: Deserialize<'de>,
            O: Serialize,
            F: Fn(&I) -> O + Send + Sync + 'static
    {
        let handler_wrapper = move |input_data: &str| {
            let input_value = serde_json::from_str(input_data);
            if input_value.is_err() {
                log::error!("Invalid input req: {}", input_data);
            }
            let input_value: I = input_value.unwrap();
            let output_value = handler(&input_value);
            let output_data = serde_json::to_string(&output_value).unwrap();
            return output_data;
        };

        let listener = Listener {
            handler: Box::new(handler_wrapper),
        };

        self.add_raw_listener(key, listener);
    }

    pub fn on_generic_call_async<I, O, F>(&self, key: &str, handler: F) -> AsyncHandler<I, O> where
            for<'de> I: Deserialize<'de> + Send + 'static,
            O: Serialize + Send + 'static,
            F: Fn(&I) -> O + 'static
    {
        let (request_tx, request_rx) = std::sync::mpsc::sync_channel(1);
        let (response_tx, response_rx) = std::sync::mpsc::sync_channel(1);

        let mpsc_mutex = Mutex::new((request_tx, response_rx));

        let handler_wrapper = move |input_data: &str| {
            let mpsc_channel = mpsc_mutex.lock().unwrap();
            let input_value: I = serde_json::from_str(input_data).unwrap();
            let (tx, rx) = mpsc_channel.deref();
            tx.send(input_value).unwrap();
            let output_value: O = rx.recv().unwrap();
            let output_data = serde_json::to_string(&output_value).unwrap();
            return output_data;
        };

        let listener = Listener {
            handler: Box::new(handler_wrapper),
        };

        self.add_raw_listener(key, listener);

        AsyncHandler {
            request_rx,
            response_tx,
            handler: Box::new(handler),
        }
    }

    pub fn get_generic_call_async_receiver<I, O>(&self, key: &str) -> RequestAsyncReceiver<I, O> where
            for<'de> I: Deserialize<'de> + Send + 'static,
            O: Serialize + Send + 'static,
    {
        let (request_tx, request_rx) = std::sync::mpsc::sync_channel(1);
        let (response_tx, response_rx) = std::sync::mpsc::sync_channel(1);

        let mpsc_mutex = Mutex::new((request_tx, response_rx));

        let handler_wrapper = move |input_data: &str| {
            let mpsc_channel = mpsc_mutex.lock().unwrap();
            let input_value: I = serde_json::from_str(input_data).unwrap();
            let (tx, rx) = mpsc_channel.deref();
            tx.send(input_value).unwrap();
            let output_value: O = rx.recv().unwrap();
            let output_data = serde_json::to_string(&output_value).unwrap();
            return output_data;
        };

        let listener = Listener {
            handler: Box::new(handler_wrapper),
        };

        self.add_raw_listener(key, listener);

        RequestAsyncReceiver {
            request_rx,
            response_tx,
            pending_response: Cell::new(false),
        }
    }

    fn add_raw_listener(&self, key: &str, listener: Listener) {
        let mut calls = self.calls.write().unwrap();
        calls.insert(key.to_string(), listener);
    }

    fn call_raw(&self, key: &str, input_data: &str) -> String {
        let calls = self.calls.read().unwrap();
        return if let Some(listener) = calls.get(key) {
            let handler = listener.handler.deref();
            handler(input_data)
        } else {
            String::from("{ }")
        }
    }

    pub fn add_get_file_handler<F>(&self, key: &str, handler: F) where
            F: Fn(&str) -> Result<Vec<u8>, std::io::Error> + Send + Sync + 'static
    {
        let listener = GetFileListener {
            handler: Box::new(handler),
        };

        self.add_raw_get_file_listener(key, listener);
    }

    fn add_raw_get_file_listener(&self, key: &str, listener: GetFileListener) {
        let mut get_file_calls = self.get_file_calls.write().unwrap();
        get_file_calls.insert(key.to_string(), listener);
    }

    fn get_file(&self, key: &str, path: &str) -> Result<Vec<u8>, std::io::Error> {
        let file_calls = self.get_file_calls.read().unwrap();
        return if let Some(listener) = file_calls.get(key) {
            let handler = listener.handler.deref();
            handler(path)
        } else {
            Ok(Vec::new())
        }
    }

}

impl ServiceApi for Rpc {

}

pub struct RpcGate {
    rpc: Arc<Rpc>,
}

impl RpcGate {

    pub fn call_raw(&self, key: &str, input_data: &str) -> String {
        return self.rpc.call_raw(key, input_data);
    }

    pub fn get_file(&self, key: &str, path: &str) -> Result<Vec<u8>, std::io::Error> {
        return self.rpc.get_file(key, path)
    }

}

impl ServiceApi for RpcGate {

}

impl ServiceInitializer for Rpc {
    fn initialize(context: &Context) -> Arc<Self> {
        let service = Arc::new(Self::new());
        let gate = RpcGate {
            rpc: service.clone(),
        };
        context.add_service(gate);
        return service;
    }
}

#[macro_export]
macro_rules! register_rpc_handler {
    ($rpc:expr, $service:expr, $key:expr, $method:ident ($($arg_name:ident : $arg_type:ty),*)) => {
        #[allow(unused_variables)]
        {
            let service_copy = $service.clone();

            #[derive(serde::Deserialize)]
            struct Args {
                #[allow(dead_code)]
                pub value: Option<i32>,
                $($arg_name : $arg_type),*
            }

            $rpc.on_generic_call_fn($key, move |args: &Args| {
                service_copy.$method($(args.$arg_name.clone()),*)
            });
        }
    };
}
