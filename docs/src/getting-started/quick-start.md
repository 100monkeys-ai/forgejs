# Quick Start

This guide creates a new Forge application from scratch and walks through the generated project.

## Create a New App

```sh
forge new my-app
cd my-app
forge dev
```

`forge new` scaffolds a complete project with a working home page, an example data schema, and the forge.toml manifest. `forge dev` starts the development server with hot reload.

Open `http://localhost:3000` in your browser.

## The Generated Project

After `forge new my-app`, your project looks like this:

```
my-app/
├── forge.toml          # Project manifest
├── foundry.lock        # Foundry lockfile (commit this)
├── app/
│   ├── routes.fx       # Route definitions
│   ├── schema.fx       # Database schema
│   ├── auth.fx         # Auth configuration
│   ├── pages/
│   │   ├── Home.fx     # Home page component
│   │   └── Layout.fx   # Root layout
│   └── server/
│       └── posts.fx    # Example server functions
└── public/
    └── favicon.ico
```

## Initial File Contents

`app/routes.fx` — the application's URL structure:

```typescript
import { Router } from 'forge:router'

export default Router.define([
  { path: '/', component: () => import('./pages/Home.fx') },
])
```

`app/schema.fx` — the database schema:

```typescript
import { Schema } from 'forge:data'

export const Posts = Schema.table('posts', {
  id: Schema.id(),
  title: Schema.text().notNull(),
  body: Schema.text().notNull(),
  createdAt: Schema.timestamp().default('now'),
})
```

`app/pages/Home.fx` — the home page:

```typescript
"use module client"

import { Link } from 'forge:router'

export default component Home() {
  return (
    <main>
      <h1>Welcome to Forge</h1>
      <p>Edit <code>app/pages/Home.fx</code> to get started.</p>
      <Link href="/about">About</Link>
    </main>
  )
}
```

## Next Steps

- Add routes in `app/routes.fx`
- Define your schema in `app/schema.fx` and run `forge migrate`
- Write server functions in `app/server/`
- Configure authentication in `app/auth.fx`
