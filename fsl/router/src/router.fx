// Router — the root routing primitive.
//
// Forge uses explicit route definitions rather than file-system-based routing.
// This is a deliberate choice: file-system routing is "magic" that makes it
// hard to understand the application's URL structure without running it.
// Explicit routes are code — readable, refactorable, testable.
//
// The Router reads the route tree and:
// 1. Renders the matching route component for the current URL
// 2. Generates the server-side route table for the HTTP router
// 3. Feeds the code splitter with per-route chunk boundaries
// 4. Generates prefetch hints for route components

"use module client"

export interface RouteDefinition {
  path: string
  component: () => Promise<{ default: unknown }>
  auth?: boolean
  layout?: () => Promise<{ default: unknown }>
}

export const Router = {
  define(routes: RouteDefinition[]) {
    // Compiler processes this at build time to generate the route manifest
    return routes
  }
}
