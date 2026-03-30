# Routing

Forge uses explicit route definitions. Routes are declared in `app/routes.fx` using `Router.define()` from `forge:router`. The compiler reads the route tree at build time to generate the HTTP router table, split code bundles per route, and produce prefetch hints.

## Defining Routes

```typescript
// app/routes.fx
import { Router } from 'forge:router'

export default Router.define([
  { path: '/', component: () => import('./pages/Home.fx') },
  { path: '/about', component: () => import('./pages/About.fx') },
  { path: '/users/:id', component: () => import('./pages/UserDetail.fx') },
  { path: '/blog/:slug', component: () => import('./pages/BlogPost.fx') },
])
```

Each route is an object with:

- `path` — the URL pattern. Supports `:param` named parameters and `*` wildcards.
- `component` — a dynamic import returning the page component. Used as a code-split boundary.
- `auth` — set to `true` to require authentication. Unauthenticated users are redirected to the login page.
- `layout` — an optional layout component that wraps the page. Nested layouts compose automatically.

## Route Parameters

Access route parameters inside a component using `useParams()`:

```typescript
import { useParams } from 'forge:router'

export default component UserDetail() {
  const { id } = useParams<{ id: string }>()
  // ...
}
```

## Nested Layouts

Layouts wrap their route's component. Multiple routes can share a layout:

```typescript
Router.define([
  {
    path: '/dashboard',
    component: () => import('./pages/Dashboard.fx'),
    layout: () => import('./layouts/AppLayout.fx'),
  },
  {
    path: '/dashboard/settings',
    component: () => import('./pages/Settings.fx'),
    layout: () => import('./layouts/AppLayout.fx'),
  },
])
```

The layout component receives a `children` prop which is the rendered page:

```typescript
export default component AppLayout({ children }: { children: unknown }) {
  return (
    <div class="app-shell">
      <Sidebar />
      <main>{children}</main>
    </div>
  )
}
```

## Protected Routes

Set `auth: true` on a route to require authentication:

```typescript
{ path: '/dashboard', component: () => import('./pages/Dashboard.fx'), auth: true }
```

The redirect destination is configured in `app/auth.fx` via `loginPath`. Defaults to `/login`.

## Navigation

Use the `<Link>` component for client-side navigation:

```typescript
import { Link } from 'forge:router'

<Link href="/users/42">View Profile</Link>
```

For programmatic navigation, use `navigate()`:

```typescript
import { navigate } from 'forge:router'

navigate('/dashboard', { replace: true })
```

## Catch-All Routes

```typescript
{ path: '/docs/*', component: () => import('./pages/Docs.fx') }
```

The matched wildcard is available as `params['*']`.
