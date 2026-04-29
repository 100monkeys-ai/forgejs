//! The core V8/deno_core runtime wrapper.
//!
//! [`ForgeRuntime`] wraps `deno_core::JsRuntime` with Forge-specific
//! initialization: registering the WinterTC op set, loading the signals
//! polyfill, and configuring the module loader.
//!
//! ## Embedding Model
//!
//! For the self-contained server binary target, compiled JavaScript is
//! embedded as static bytes in the binary at compile time (via `include_bytes!`
//! or a build script). The module loader serves these bytes directly without
//! reading from disk, making the binary truly self-contained.
//!
//! For the development server, the module loader reads from disk and supports
//! hot module replacement via the file watcher.

use crate::error::RuntimeError;
use crate::isolate::ops;
use deno_core::JsRuntime;
use deno_core::RuntimeOptions;

/// The Forge JavaScript runtime.
///
/// Wraps `deno_core::JsRuntime` with Forge-specific initialization.
/// One `ForgeRuntime` instance exists per isolate. For the server binary,
/// a pool of runtimes handles concurrent requests.
pub struct ForgeRuntime {
    #[allow(dead_code)]
    inner: JsRuntime,
}

impl ForgeRuntime {
    /// Create a new ForgeRuntime with the standard WinterTC op set registered.
    pub fn new() -> Result<Self, RuntimeError> {
        let options = RuntimeOptions {
            extensions: vec![ops::forge_ops::init_ops_and_esm()],
            ..Default::default()
        };

        let inner = JsRuntime::new(options);

        Ok(Self { inner })
    }

    /// Execute a compiled JavaScript module and return its default export.
    pub async fn execute_module(&mut self, _module_source: &[u8]) -> Result<(), RuntimeError> {
        // TODO: Load and execute module via deno_core
        Ok(())
    }
}

impl Default for ForgeRuntime {
    fn default() -> Self {
        Self::new().expect("failed to initialize ForgeRuntime")
    }
}
