//! Source map generation for compiled JavaScript.
//!
//! Source maps allow browser DevTools and error trackers to map compiled
//! JavaScript back to the original `.fx` or `.ts` source. Forge generates
//! inline source maps in development and separate `.map` files in production.
