//! Non-JavaScript asset processing.
//!
//! The asset pipeline handles:
//!
//! - **CSS**: processed via Lightning CSS (Rust-native) for vendor prefixing,
//!   minification, and CSS Modules support
//! - **Images**: referenced images are hashed and copied to the output directory
//! - **Fonts**: same as images
//! - **Static files**: files in `app/assets/` are copied verbatim
//!
//! All asset URLs in the emitted JavaScript are replaced with content-hashed
//! paths (e.g., `logo.abc123.svg`) for optimal browser caching.
