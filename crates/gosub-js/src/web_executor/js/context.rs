use crate::web_executor::js::{JSArray, JSCompiled, JSFunction, JSObject, JSRuntime, JSValue};

//main trait for JS context (can be implemented for different JS engines like V8, SpiderMonkey, JSC, etc.)
pub trait JSContext: Clone {
    type RT: JSRuntime;
    fn run(&mut self, code: &str) -> crate::types::Result<<Self::RT as JSRuntime>::Value>;

    fn compile(&mut self, code: &str) -> crate::types::Result<<Self::RT as JSRuntime>::Compiled>;

    fn run_compiled(
        &mut self,
        compiled: &mut <Self::RT as JSRuntime>::Compiled,
    ) -> crate::types::Result<<Self::RT as JSRuntime>::Value>;

    // fn compile_stream(&self, code: &str) -> Result<()>;

    fn new_global_object(
        &mut self,
        name: &str,
    ) -> crate::types::Result<<Self::RT as JSRuntime>::Object>;
}
