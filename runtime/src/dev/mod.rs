//! Development server with hot module replacement (HMR).
//!
//! The dev server extends the production HTTP server with:
//!
//! - **File watching**: monitors the source directory for changes
//! - **Incremental recompilation**: only recompiles changed modules
//! - **Hot module replacement**: pushes updates to connected browsers
//!   without a full page reload
//! - **Forge Studio**: serves the Studio UI on a separate port
//!
//! The dev server is compiled into the `forge` binary and activated by
//! the `forge dev` command. It is never present in production builds.

pub mod dev_server;
pub mod hot_reload;
