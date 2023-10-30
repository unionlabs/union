use wasmer::Module;

#[derive(Debug, Clone)]
pub struct SizedModule {
    pub module: Module,
    pub size: usize,
}
