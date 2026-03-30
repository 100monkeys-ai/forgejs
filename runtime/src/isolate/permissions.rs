//! Runtime permission model for Forge ops.
//!
//! Forge's permission model is inspired by Deno's but simplified for the
//! framework use case. Rather than per-invocation permission prompts, Forge
//! uses a static permission set determined at startup by the deployment target:
//!
//! - **Server binary**: full WinterTC API set, plus filesystem access to the
//!   configured data directory
//! - **Edge**: fetch + crypto + streams only (no filesystem, no subprocess)
//! - **Development**: all APIs plus dev-specific ops (HMR, Studio)
//!
//! The permission set is fixed at runtime initialization and cannot be
//! escalated from JavaScript.

/// The set of permissions granted to a ForgeRuntime instance.
#[derive(Debug, Clone)]
pub struct PermissionSet {
    pub allow_fetch: bool,
    pub allow_filesystem: bool,
    pub allow_subprocess: bool,
    pub allow_env: bool,
    pub allow_dev_ops: bool,
}

impl PermissionSet {
    /// Full permissions for the development server.
    pub fn development() -> Self {
        Self {
            allow_fetch: true,
            allow_filesystem: true,
            allow_subprocess: false,
            allow_env: true,
            allow_dev_ops: true,
        }
    }

    /// Restricted permissions for edge deployment.
    pub fn edge() -> Self {
        Self {
            allow_fetch: true,
            allow_filesystem: false,
            allow_subprocess: false,
            allow_env: false,
            allow_dev_ops: false,
        }
    }

    /// Standard permissions for the production server binary.
    pub fn server() -> Self {
        Self {
            allow_fetch: true,
            allow_filesystem: true,
            allow_subprocess: false,
            allow_env: true,
            allow_dev_ops: false,
        }
    }
}
