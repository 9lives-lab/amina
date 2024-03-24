use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};
use std::sync::atomic::{AtomicUsize, Ordering};

use futures::SinkExt;
use tokio::runtime;
use tokio::sync::{mpsc};
use bytes::Bytes;
use warp::{Filter, reply, Rejection, Reply};
use warp::path::Tail;
use warp::ws::{Message, WebSocket};

use amina_core::events::EventEmitterGate;
use amina_core::rpc::RpcGate;
use amina_core::service::{Context, Service};

struct WsUsers {
    next_id: AtomicUsize,
    users: RwLock<HashMap<usize, mpsc::UnboundedSender<Message>>>,
}

pub struct EventToUi {
    pub key: String,
    pub data: String,
}

pub struct RpcServer {
    _rt: runtime::Runtime,
}

impl RpcServer {
    pub fn run(context: &Context) -> Self {
        let users = Arc::new(WsUsers {
            next_id: AtomicUsize::new(1),
            users: RwLock::default(),
        });

        let rpc_gate = context.get_service::<RpcGate>();
        let events_gate = context.get_service::<EventEmitterGate>();

        let users_copy = users.clone();
        events_gate.add_raw_observer(Box::new(move |key: &str, raw_value: &str| {
            let users_vec = users_copy.users.read().unwrap();
            for (_, user_id) in users_vec.iter() {
                let msg = format!("{{\"key\":\"{ }\", \"data\":{ } }}", key, raw_value);
                let msg = Message::text(msg);
                if let Err(e) = user_id.send(msg.clone()) {
                    log::trace!("Send error: {:?}", e);
                }
            }
        }));

        let rpc_gate_filter = warp::any().map(move || rpc_gate.clone()).boxed();

        let cors = warp::cors()
            .allow_any_origin()
            .allow_methods(&[warp::http::Method::GET, warp::http::Method::POST])
            .allow_headers(vec![
                "Origin",
                "Content-Type",
                "Accept",
                "Authorization",
            ])
            .max_age(3600).build();

        let prc_call_handler = warp::post()
            .and(warp::path!("api" / "rpc_call"))
            .and(rpc_gate_filter.clone())
            .and(warp::query::<HashMap<String, String>>())
            .and(warp::body::bytes())
            .and_then(handle_rpc_call)
            .with(cors.clone());

        let get_file_handler = warp::get()
            .and(warp::path("get_file"))
            .and(rpc_gate_filter.clone())
            .and(warp::path::tail())
            .and_then(handle_get_file);

        let users_copy = users.clone();
        let events_ws_handler = warp::path!("api" / "events")
            .and(warp::ws())
            .map(move |ws: warp::ws::Ws| {
                let users_copy = users_copy.clone();
                ws.on_upgrade(move |socket|
                    Self::user_connected(socket, users_copy.clone())
                )
            });

        let rt = runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .unwrap();

        let addr = SocketAddr::from(([127, 0, 0, 1], 8090));

        rt.spawn(async move {
            warp::serve(prc_call_handler.or(events_ws_handler).or(get_file_handler))
                .run(addr)
                .await;
        });

        RpcServer {
            _rt: rt,
        }
    }

    pub fn stop(&self) {
        log::info!("Stop server");
    }

    async fn user_connected(mut ws: WebSocket, ws_users: Arc<WsUsers>) {
        let user_id = ws_users.next_id.fetch_add(1, Ordering::Relaxed);

        let (tx, mut rx) = mpsc::unbounded_channel();

        ws_users.users.write().unwrap().insert(user_id, tx);

        while let Some(message) = rx.recv().await {
            let result = ws.send(message).await;
            if let Err(e) = result {
                log::trace!("ws send error: {:?}", e);
                break;
            }
        }

        ws_users.users.write().unwrap().remove(&user_id);
    }
}

async fn handle_rpc_call(rpc_gate: Service<RpcGate>, p: HashMap<String, String>, bytes: Bytes) -> Result<impl Reply, Rejection> {
    match p.get("key") {
        Some(key) => {
            let request = String::from_utf8(bytes.to_vec()).unwrap();
            let key = key.clone();
            let response = tokio::task::spawn_blocking(move || {
                rpc_gate.call_raw(&key, request.as_str())
            }).await.unwrap();
            let response = reply::with_header(response, "Content-Type", "application/json");
            Ok(reply::with_status(response, warp::http::StatusCode::OK))
        },
        None => Ok(reply::with_status(
            reply::with_header(String::from("No \"key\" param in query."), "Content-Type", "application/json"),
            warp::http::StatusCode::BAD_REQUEST)),
    }
}

async fn handle_get_file(rpc_gate: Service<RpcGate>, tail: Tail) -> Result<impl Reply, Rejection> {
    let key_value: Vec<&str> = tail.as_str().splitn(2, "/").collect();
    let key = key_value[0];
    let path = key_value[1];
    let file_bytes =  rpc_gate.get_file(key, path);
    match file_bytes {
        Ok(file_bytes) => {
            let response = warp::http::Response::builder()
                .body(file_bytes)
                .unwrap();
            Ok(reply::with_status(response, warp::http::StatusCode::OK))
        },
        Err(e) => {
            log::error!("Error: {:?}", e);
            Err(warp::reject::not_found())
        }
    }
}

