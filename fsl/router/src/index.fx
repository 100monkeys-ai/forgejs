// forge:router — Forge Standard Library routing module
//
// Provides the Router, Link, and navigation primitives for Forge applications.
// Routes are defined explicitly in app/routes.fx using Router.define().
//
// Example:
//   import { Router } from 'forge:router'
//   export default Router.define([
//     { path: '/', component: () => import('./pages/Home.fx') },
//     { path: '/users/:id', component: () => import('./pages/UserDetail.fx') },
//   ])

export { Router } from './router.fx'
export { Link } from './link.fx'
export { navigate, useParams, useLocation } from './navigate.fx'
export type { RouteDefinition, RouterConfig, Params } from './types'
