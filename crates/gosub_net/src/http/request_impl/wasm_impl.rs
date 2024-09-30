use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use anyhow::anyhow;
use js_sys::{ArrayBuffer, Promise, Uint8Array};
use wasm_bindgen_futures::JsFuture;
use web_sys::{RequestInit, RequestMode};
use web_sys::wasm_bindgen::JsCast;
use gosub_shared::types::Result;
use gosub_shared::worker::WasmWorker;
use crate::http::fetcher::RequestAgent;
use crate::http::headers::Headers;
use crate::http::request::Request;
use crate::http::response::Response;

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

    fn get(&self, url: &str) -> Result<Response> {

        let mut worker = WasmWorker::new()?;
        
        let url = url.to_string();

        // This is safe to do, because we only move the future once to the other thread and then never again
        //Our captures are also Send (url)
        Ok(worker.async_blocking_return(UnsafeFuture::from(|| async move {
            let opts = RequestInit::new();

            opts.set_method("GET");
            opts.set_mode(RequestMode::Cors);

            let req = web_sys::Request::new_with_str_and_init(&url, &opts)
                .map_err(|e| anyhow!(e.as_string().unwrap_or("<unknown>".into())))?;

            let res = fetch(req).await?;
            
            Result::Ok(res)
        }))??)
    }

    fn get_req(
        &self,
        req: &Request,
    ) -> Result<Response> {

        let mut worker = WasmWorker::new()?;
        
        let req = req.clone();

        // This is safe to do, because we only move the future once to the other thread and then never again
        //Our captures are also Send (req)
        Ok(worker.async_blocking_return(UnsafeFuture::from(|| async move {
            let opts = RequestInit::new();

            opts.set_method(&req.method);
            opts.set_mode(RequestMode::Cors);

            opts.set_body(&req.body.clone().into());

            //TODO: headers, version, cookies

            let req = web_sys::Request::new_with_str_and_init(&req.uri, &opts)
                .map_err(|e| anyhow!(e.as_string().unwrap_or("<unknown>".into())))?;

            fetch(req).await
        }))??)




    }
}

struct UnsafeFuture<F: FnOnce() -> Fut, Fut: Future> {
    inner: F,
}


impl<F: FnOnce() -> Fut, Fut: Future> From<F> for UnsafeFuture<F, Fut> {
    fn from(inner: F) -> Self {
        Self {
            inner,
        }
    }
}


/// Generally this is NOT safe to do, but in this context, 
unsafe impl<F: FnOnce() -> Ret + Future, Ret> Send for UnsafeFuture<F, Ret> {}


impl<F: FnOnce() -> Fut, Fut: Future> Future for UnsafeFuture<F, Fut> {
    type Output =  Fut::Output;


    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        
        let pin = unsafe { Pin::new_unchecked(&mut self.get_mut().inner) };
        
        
        Fut::poll(pin, cx)
    }

}


async fn fetch(req: web_sys::Request) -> Result<Response> {
    let window = web_sys::window().ok_or(anyhow!("No window"))?;

    
    let resp = JsFuture::from(window.fetch_with_request(&req)).await
        .map_err(|e| anyhow!(e.as_string().unwrap_or("<unknown>".into())))?;

    let resp: web_sys::Response = resp.dyn_into()
        .map_err(|e| anyhow!(e.as_string().unwrap_or("<unknown>".into())))?;

    // let req_headers = resp.headers();

    let headers = Headers::new();


    // for iter in req_headers.values() {
    //
    //     let iter_value = iter?;
    //
    //
    //
    //
    //     headers.append(&name, &value);
    // }



    let cookies = Default::default();



    let buf = JsFuture::from(
        resp.array_buffer()
            .map_err(|e| anyhow!(e.as_string().unwrap_or("<unknown>".into())))?
    ).await
        .map_err(|e| anyhow!(e.as_string().unwrap_or("<unknown>".into())))?;


    let array: ArrayBuffer = buf.dyn_into()
        .map_err(|e| anyhow!(e.as_string().unwrap_or("<unknown>".into())))?;

    let body = Uint8Array::new(&array).to_vec();


    Ok(Response {
        status: resp.status(),
        status_text: resp.status_text(),
        version: Default::default(),
        headers,
        cookies,
        body,
    })
}
