use std::sync::{Arc, RwLock};
use std::ops::Deref;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::service::{ServiceApi, ServiceInitializer, Context, Service};
use crate::tasks::TaskManager;

#[derive(Serialize, Deserialize)]
pub struct EmptyEvent {
    _dummy: Option<i32>,
}

impl EmptyEvent {
    pub fn new() -> Self {
        Self {
            _dummy: None,
        }
    }
}

pub trait Event: Send + Sync {
    fn get_key() -> &'static str;
}

pub struct Listener {
    handler: Box<dyn Fn(&str) + Sync + Send + 'static>,
}

pub struct EventEmitter {
    events: RwLock<HashMap<String, Vec<Listener>>>,
    observers: RwLock<Vec<Box<dyn Fn(&str, &str) + Sync + Send + 'static>>>,
    task_manager: Service<TaskManager>,
}

impl EventEmitter {

    pub fn on_generic_event_fn<E, F>(&self, key: &str, handler: F) where
            for<'de> E: Deserialize<'de> + Send + Sync + 'static,
            F: Fn(&E) + Send + Sync + 'static
    {
        let task_manager = self.task_manager.clone();
        let handler = Arc::new(handler);
        let handler_wrapper = move |event_data: &str| {
            let value: E = serde_json::from_str(event_data).unwrap();
            let handler_clone = handler.clone();
            task_manager.run_instant_task(move |_| {
                handler_clone(&value);
            });
        };

        let listener = Listener {
            handler: Box::new(handler_wrapper),
        };

        self.add_raw_listener(key, listener);
    }

    pub fn on_event_fn<E, F>(&self, handler: F) where
            for<'de> E: Event + Deserialize<'de> + 'static,
            F: Fn(&E) + Send + Sync + 'static
    {
        self.on_generic_event_fn(E::get_key(), handler);
    }

    pub fn emit<T>(&self, key: &str, value: &T) where
        T: Serialize
    {
        let event_data = serde_json::to_string(value).unwrap();
        self.send_raw_event(key, &event_data);
        self.send_to_observers(key, &event_data)
    }

    pub fn emit_event<E>(&self, value: &E) where
        E: Event + Serialize
    {
        let event_data = serde_json::to_string(value).unwrap();
        let key = E::get_key();
        self.send_raw_event(key, &event_data);
        self.send_to_observers(key, &event_data)
    }

    fn add_raw_listener(&self, key: &str, listener: Listener) {
        let mut events = self.events.write().unwrap();
        match events.get_mut(key) {
            Some(handlers) => {
                handlers.push(listener);
            },
            None => {
                events.insert(key.to_string(), vec![listener]);
            }
        };
    }

    fn send_raw_event(&self, key: &str, event_data: &str) {
        let events = self.events.read().unwrap();
        if let Some(listeners) = events.get(key) {
            for listener in listeners.iter() {
                let handler = listener.handler.deref();
                handler(event_data);
            }
        }
    }

    fn add_raw_observer(&self, observer: Box<dyn Fn(&str, &str) + Sync + Send + 'static>) {
        let mut observers = self.observers.write().unwrap();
        observers.push(observer);
    }

    fn send_to_observers(&self, key: &str, event_data: &str) {
        let observers = self.observers.read().unwrap();
        for observer in observers.iter() {
            let handler = observer.deref();
            handler(key, event_data);
        }
    }

}

impl ServiceApi for EventEmitter {

}

pub struct EventEmitterGate {
    event_emitter: Arc<EventEmitter>,
}

impl EventEmitterGate {

    pub fn send_raw_event(&self, key: &str, event_data: &str) {
        self.event_emitter.send_raw_event(key, event_data);
    }

    pub fn add_raw_observer(&self, observer: Box<dyn Fn(&str, &str) + Sync + Send + 'static>) {
        self.event_emitter.add_raw_observer(observer);
    }

}

impl ServiceApi for EventEmitterGate {

}

impl ServiceInitializer for EventEmitter {
    fn initialize(context: &Context) -> Arc<Self> {
        let task_manager = context.get_service::<TaskManager>();
        let service = Arc::new(Self {
            events: RwLock::new(HashMap::new()),
            observers: RwLock::new(Vec::new()),
            task_manager,
        });
        let gate = EventEmitterGate {
            event_emitter: service.clone(),
        };
        context.add_service(gate);
        return service;
    }
}

#[macro_export]
macro_rules! register_event_handler {
    ($event_emitter:expr, $service:expr, $method:ident) => {
        {
            let service_copy = $service.clone();

            $event_emitter.on_event_fn(move |event| {
                service_copy.$method(event)
            });
        }
    };
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};
    use std::time::Duration;
    use serde::{Deserialize, Serialize};
    use amina_core_derive::Event;
    use crate::service::{ServiceApi, Context, ServiceInitializer};
    use crate::events::{Event, EventEmitter};
    use crate::tasks::TaskManager;

    #[derive(Serialize, Deserialize)]
    #[derive(Event)]
    #[key = "event.one"]
    struct EventOne {
        value: String,
    }

    #[derive(Serialize, Deserialize)]
    #[derive(Event)]
    #[key = "event.second"]
    struct EventSecond {
        value: String,
    }

    struct ServiceWithCallback {
        event_one_tx: std::sync::mpsc::SyncSender<String>,
        event_second_tx: std::sync::mpsc::SyncSender<String>,
        event_one_rx: Mutex<std::sync::mpsc::Receiver<String>>,
        event_second_rx: Mutex<std::sync::mpsc::Receiver<String>>,
    }

    impl ServiceWithCallback {
        fn on_event_one(&self, event: &EventOne) {
            println!("EventOne get: {}", event.value.as_str());
            self.event_one_tx.send(event.value.clone()).unwrap();
        }

        fn on_event_second(&self, event: &EventSecond) {
            println!("EventSecond get: {}", event.value.as_str());
            self.event_second_tx.send(event.value.clone()).unwrap();
        }

        fn get_event_one_data(&self) -> String {
            let event_one_data = self.event_one_rx.lock().unwrap().recv_timeout(Duration::from_secs(1)).unwrap();
            event_one_data
        }

        fn get_event_second_data(&self) -> String {
            let event_second_data = self.event_second_rx.lock().unwrap().recv_timeout(Duration::from_secs(1)).unwrap();
            event_second_data
        }
    }

    impl ServiceApi for ServiceWithCallback {

    }

    impl ServiceInitializer for ServiceWithCallback {
        fn initialize(context: &Context) -> Arc<Self> {
            let event_emitter = context.get_service::<EventEmitter>();

            let (event_one_tx, event_one_rx) = std::sync::mpsc::sync_channel(1);
            let (event_second_tx, event_second_rx) = std::sync::mpsc::sync_channel(1);
            
            let service = Arc::new(Self {
                event_one_tx,
                event_second_tx,
                event_one_rx: Mutex::new(event_one_rx),
                event_second_rx: Mutex::new(event_second_rx),
            });

            register_event_handler!(event_emitter, service, on_event_one);
            register_event_handler!(event_emitter, service, on_event_second);

            service
        }
    }

    #[test]
    fn test_basic() {
        let context = Context::new();

        context.init_service::<TaskManager>();
        context.init_service::<EventEmitter>();
        context.init_service::<ServiceWithCallback>();

        let service = context.get_service::<ServiceWithCallback>();
        let event_emitter = context.get_service::<EventEmitter>();

        event_emitter.emit(EventOne::get_key(), &EventOne {
            value: "value 1".to_string(),
        });
        assert_eq!(service.get_event_one_data(), "value 1".to_string());

        event_emitter.emit_event(&EventSecond {
            value: "value 2".to_string(),
        });
        assert_eq!(service.get_event_second_data(), "value 2".to_string());
    }

}
