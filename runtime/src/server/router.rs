//! Request routing: maps incoming HTTP requests to handlers.
//!
//! The router is built from the compiled route manifest produced by the
//! compiler. For each route, the router knows:
//!
//! - The URL pattern (e.g., `/users/:id`)
//! - Whether the route requires authentication
//! - The server-side rendering function for this route
//! - The server functions registered at this route's RPC paths
