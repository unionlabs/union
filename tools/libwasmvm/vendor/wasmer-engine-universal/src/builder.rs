use crate::UniversalEngine;
use wasmer_compiler::{CompilerConfig, Features, Target};

/// The Universal builder
pub struct Universal {
    #[allow(dead_code)]
    compiler_config: Option<Box<dyn CompilerConfig>>,
    target: Option<Target>,
    features: Option<Features>,
}

impl Universal {
    /// Create a new Universal
    pub fn new<T>(compiler_config: T) -> Self
    where
        T: Into<Box<dyn CompilerConfig>>,
    {
        Self {
            compiler_config: Some(compiler_config.into()),
            target: None,
            features: None,
        }
    }

    /// Create a new headless Universal
    pub fn headless() -> Self {
        Self {
            compiler_config: None,
            target: None,
            features: None,
        }
    }

    /// Set the target
    pub fn target(mut self, target: Target) -> Self {
        self.target = Some(target);
        self
    }

    /// Set the features
    pub fn features(mut self, features: Features) -> Self {
        self.features = Some(features);
        self
    }

    /// Build the `UniversalEngine` for this configuration
    #[cfg(feature = "compiler")]
    pub fn engine(self) -> UniversalEngine {
        let target = self.target.unwrap_or_default();
        if let Some(compiler_config) = self.compiler_config {
            let features = self
                .features
                .unwrap_or_else(|| compiler_config.default_features_for_target(&target));
            let compiler = compiler_config.compiler();
            UniversalEngine::new(compiler, target, features)
        } else {
            UniversalEngine::headless()
        }
    }

    /// Build the `UniversalEngine` for this configuration
    #[cfg(not(feature = "compiler"))]
    pub fn engine(self) -> UniversalEngine {
        UniversalEngine::headless()
    }
}
