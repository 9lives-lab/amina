use serde::Serialize;
use tauri::{AppHandle, Emitter};

use amina_core::events::EventEmitterGate;
use amina_core::rpc::RpcGate;
use amina_core::service::Service;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct AminaEvent {
  key: String,
  data: String,
}

#[tauri::command]
pub fn rpc_handler(rpc_gate: tauri::State<Service<RpcGate>>, key: String, request: String) -> String {
    rpc_gate.call_raw(&key, &request)
}

pub fn setup_events_gate(app_handle: AppHandle, events_gate: Service<EventEmitterGate>) {
    events_gate.add_raw_observer(Box::new(move |key: &str, raw_value: &str| {
        let payload = AminaEvent {
            key: key.to_string(),
            data: raw_value.to_string(),
        };
        app_handle.emit("amina-event", payload).unwrap();
    }));
}

