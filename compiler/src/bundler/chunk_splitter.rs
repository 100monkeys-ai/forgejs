//! Route-based code splitting.
//!
//! Each route in the application is a natural code splitting boundary.
//! Code that is only needed for a specific route is placed in a separate
//! chunk that is loaded lazily when the user navigates to that route.
//!
//! The chunk splitter reads the route graph from the route compiler and
//! uses it to partition the module graph into per-route chunks plus a
//! shared chunk for code used by multiple routes.
