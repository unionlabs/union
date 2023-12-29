// Allow unused imports while developing
#![allow(unused_imports, dead_code)]

use crate::compiler::SinglepassCompiler;
use std::sync::Arc;
use wasmer_compiler::{Compiler, CompilerConfig, Engine, EngineBuilder, ModuleMiddleware};
use wasmer_types::{CpuFeature, Features, Target};

#[derive(Debug, Clone)]
pub struct Singlepass {
    pub(crate) enable_nan_canonicalization: bool,
    /// The middleware chain.
    pub(crate) middlewares: Vec<Arc<dyn ModuleMiddleware>>,
}

impl Singlepass {
    /// Creates a new configuration object with the default configuration
    /// specified.
    pub fn new() -> Self {
        Self {
            enable_nan_canonicalization: true,
            middlewares: vec![],
        }
    }

    pub fn canonicalize_nans(&mut self, enable: bool) -> &mut Self {
        self.enable_nan_canonicalization = enable;
        self
    }
}

impl CompilerConfig for Singlepass {
    fn enable_pic(&mut self) {
        // Do nothing, since singlepass already emits
        // PIC code.
    }

    /// Transform it into the compiler
    fn compiler(self: Box<Self>) -> Box<dyn Compiler> {
        Box::new(SinglepassCompiler::new(*self))
    }

    /// Gets the default features for this compiler in the given target
    fn default_features_for_target(&self, _target: &Target) -> Features {
        let mut features = Features::default();
        features.multi_value(false);
        features
    }

    /// Pushes a middleware onto the back of the middleware chain.
    fn push_middleware(&mut self, middleware: Arc<dyn ModuleMiddleware>) {
        self.middlewares.push(middleware);
    }
}

impl Default for Singlepass {
    fn default() -> Singlepass {
        Self::new()
    }
}

impl From<Singlepass> for Engine {
    fn from(config: Singlepass) -> Self {
        EngineBuilder::new(config).engine()
    }
}
