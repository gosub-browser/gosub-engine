use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};

use js_sys::wasm_bindgen::JsValue;
use web_sys::Worker;

use crate::types::Result;

pub struct WasmWorker {
    worker: Worker,
}


pub struct Work {
    f: Box<dyn FnOnce() + Send + 'static>,
}


impl WasmWorker {
    pub fn new() -> Result<Self> {
        let worker = Worker::new("worker.js")?;
        Ok(Self {
            worker,
        })
    }

    pub fn execute(&mut self, f: impl FnOnce() + Send + 'static) -> Result<()> {
        let work = Box::new(Work {
            f: Box::new(f),
        });

        let ptr = Box::into_raw(work) as *mut _ as usize;

        self.worker.post_message(&JsValue::from(ptr))?;

        Ok(())
    }
    
    pub fn execute_blocking(&mut self, f: impl FnOnce() + Send + 'static) -> Result<()> {
        let block = Arc::new(AtomicBool::new(false));
        
        let block_clone = block.clone();

        let closure = || {
            f();
            
            block_clone.store(true, Ordering::Relaxed);
        };
        
        
        let work = Box::new(Work {
            f: Box::new(closure),
        });
        
        let ptr = Box::into_raw(work) as *mut _ as usize;
        
        self.worker.post_message(&JsValue::from(ptr))?;
        
        
        
        while block.load(Ordering::Relaxed) {
            std::hint::spin_loop();
        }
        
        Ok(())
        
    }


    pub fn execute_blocking_return<R>(&mut self, f: impl FnOnce() -> R + Send + 'static) -> Result<R> {
        let res = Arc::new(Mutex::new(None));
        let block = Arc::new(AtomicU8::new(0));
        
        // 0 = block
        // 1 = finished
        // 2 = lock error
        
        
        let block_clone = block.clone();
        let res_clone = res.clone();

        let closure = || {
            let res = f();
            
            let Ok(lock) = res_clone.lock() else {
                block_clone.store(2, Ordering::Relaxed);
                return;
            };
            
            *lock = Some(res);
            

            block_clone.store(1, Ordering::Relaxed);
        };
        
        
        let work = Box::new(Work {
            f: Box::new(closure),
        });
        
        let ptr = Box::into_raw(work) as *mut _ as usize;
        
        self.worker.post_message(&JsValue::from(ptr))?;
        
        
        
        while block.load(Ordering::Relaxed) == 0 {
            std::hint::spin_loop();
        }
        
        if block.load(Ordering::Relaxed) == 2 {
            return Err("Lock error".into());
        }
        
        
        let Ok(mut lock) = res.lock() else {
            return Err("Lock error".into());
        };
        
        let Some(res) = lock.take() else {
            return Err("Worker did not return".into());
        };
        
        Ok(res)
        
    }
}


#[cfg(target_arch = "wasm32")]
pub fn wasm_execute_work(ptr: usize) {
    let work = unsafe { Box::from_raw(ptr as *mut Work) };
    (work.f)();
}