//! Server-side rendering pipeline.
//!
//! SSR renders Forge components to HTML strings on the server, which are
//! then sent to the client for fast initial page loads. The client-side
//! JS hydrates the rendered HTML, connecting it to the reactive signal graph.
//!
//! ## SSR and Signals
//!
//! During SSR, components run in the server isolate. Signal reads return
//! their initial values. The rendered HTML includes a serialized snapshot
//! of the initial signal state, which the client-side hydration uses to
//! reconstruct the signal graph without a loading flash.
