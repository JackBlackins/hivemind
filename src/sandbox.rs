use wasmtime::*;

pub struct SecureSandbox {
    engine: Engine,
}

impl SecureSandbox {
    pub fn new() -> Self {

        let engine = Engine::default();
        SecureSandbox { engine }
    }

    pub fn execute_dynamic_wat(&self, wat_code: &str, a: i32, b: i32) -> Result<i32> {
        let module = Module::new(&self.engine, wat_code)?;
        
        let mut store = Store::new(&self.engine, ());
        let instance = Instance::new(&mut store, &module, &[])?;
        
        let func = instance.get_typed_func::<(i32, i32), i32>(&mut store, "math")?;
        let result = func.call(&mut store, (a, b))?;
        
        Ok(result)
    }
}