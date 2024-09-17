use crate::http::fetcher::RequestAgent;

pub struct WasmAgent;


impl RequestAgent for WasmAgent {
    fn new() -> Self {
        Self
    }

    fn get(&self, _url: &str) -> gosub_shared::types::Result<crate::http::response::Response> {
        todo!()
    }

    fn get_req(&self, _req: &crate::http::request::Request) -> gosub_shared::types::Result<crate::http::response::Response> {
        todo!()
    }
}