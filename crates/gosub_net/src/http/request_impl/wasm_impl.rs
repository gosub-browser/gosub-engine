use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

use crate::http::fetcher::RequestAgent;

#[derive(Debug)]
pub struct WasmAgent;

#[derive(Debug)]
pub struct WasmError {
    message: String,
}

impl Display for WasmError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for WasmError {}

impl RequestAgent for WasmAgent {
    type Error = WasmError;

    fn new() -> Self {
        Self
    }

    fn get(&self, _url: &str) -> gosub_shared::types::Result<crate::http::response::Response> {
        todo!()
    }

    fn get_req(
        &self,
        _req: &crate::http::request::Request,
    ) -> gosub_shared::types::Result<crate::http::response::Response> {
        todo!()
    }
}
