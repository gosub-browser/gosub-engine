#[cfg(not(target_arch = "wasm32"))]
mod ureq_impl;

#[cfg(target_arch = "wasm32")]
mod wasm_impl;