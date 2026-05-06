use wasmtime::*;

pub struct SecureSandbox {
    engine: Engine,
}

impl SecureSandbox {
    pub fn new() -> Self {
        // The Engine is the global context for Wasm compilation.
        // It holds the configuration for the sandbox.
        let engine = Engine::default();
        SecureSandbox { engine }
    }

    pub fn execute_dynamic_wat(&self, wat_code: &str, a: i32, b: i32) -> Result<i32> {
        // We compile the dynamic untrusted code into a Module.
        let module = Module::new(&self.engine, wat_code)?;
        
        let mut store = Store::new(&self.engine, ());
        let instance = Instance::new(&mut store, &module, &[])?;
        
        // We assume the AI will always name its exported function "add" for this phase
        let add_func = instance.get_typed_func::<(i32, i32), i32>(&mut store, "add")?;
        let result = add_func.call(&mut store, (a, b))?;
        
        Ok(result)
    }
}