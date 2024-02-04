use serde::{Deserialize, Serialize};
use reqwest::blocking::Client;
use reqwest::blocking::RequestBuilder;

#[derive(Clone)]
pub struct RpcTcpClient {
    client: Client,
}

impl RpcTcpClient {

    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    pub fn send_request<O, I>(&self, key: &str, request: &O) -> I where
            for<'de> I: Deserialize<'de> + Send + 'static,
            O: Serialize + Send + 'static,
    {
        self.request_builder(key)
            .json(request)
            .send().unwrap()
            .json().unwrap()
    }

    fn request_builder(&self, key: &str) -> RequestBuilder {
        self.client.post("http://127.0.0.1:8090/api/rpc_call").query(&[("key", key)])
    }

}
