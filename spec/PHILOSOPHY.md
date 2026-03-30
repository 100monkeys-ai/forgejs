# The Forge Manifesto

## The Problem: JavaScript's Wild West

npm launched in 2010. Its designers had a specific, narrow problem to solve: sharing browser JavaScript assets between developers. It was a good solution to that problem. The registry worked, the CLI worked, the `package.json` manifest was readable, and the community adopted it quickly.

Then Node.js happened, and npm was pressed into service as the dependency manager for an entirely different domain — server-side application development — without anyone stopping to ask whether it was the right tool for the job.

It was not.

The browser asset distribution model does not map to server-side dependency management. Browser assets are consumed once, at build time, and the result is a static bundle. Server-side dependencies are live code running in production. They have access to your filesystem, your environment variables, your database connections, your secrets. The threat model is completely different, and npm was designed for neither.

By 2015, a moderate Node.js project pulled in 300+ packages. By 2020, that number was 800+. Today, `create-next-app` installs over 1,000 packages to render a blank page with a logo. The `node_modules` directory for a hello-world Next.js project exceeds 300MB. Every one of those packages is code you are running in production, written by someone you have never met, under a license you have not read, with a security track record you have not checked.

The ecosystem calls this "the npm ecosystem" and treats it as a feature.

It is a bug.

Every problem that has arisen from this architecture — the 2016 left-pad incident that broke thousands of builds worldwide when one developer unpublished an 11-line package, the 2022 colors and faker sabotage by a maintainer protesting unpaid open-source labor, the SolarWinds-style supply chain attacks now routinely discovered in npm packages — these are not edge cases. They are the predictable consequences of treating untrusted third-party code as a first-class build input without cryptographic verification, without content addressing, and without meaningful API stability guarantees.

## The Cultural Root Cause

The mess is self-inflicted, and understanding how is necessary for understanding why Forge takes the position it does.

Frontend development in the 2010s exploded in complexity. The browser became a platform. Single-page applications required state management, component trees, client-side routing, and build pipelines that could transform TypeScript, handle CSS modules, and tree-shake dead code. The engineers who solved these problems were brilliant, and the solutions — React, Webpack, Babel — were real innovations.

Then those same engineers took their client-side toolchain assumptions and moved them to the server, wholesale, without interrogation.

React was designed for a browser component tree. When it moved to the server with Next.js, it brought the component tree mental model with it. The entire architecture of Next.js is React's component tree projected onto the server — Server Components, Server Actions, the App Router — all of it is React thinking applied to a domain where React's core abstractions (virtual DOM, reconciliation, hydration) are not just unnecessary but actively harmful.

The result: a framework that is simultaneously trying to be a server framework and a client framework, mediated by a compilation pipeline so complex that it has its own dedicated documentation section and generates entire conference talks explaining why your data sometimes appears and sometimes does not.

Webpack brought its own original sin. It was built to bundle browser assets. When used for server builds, you get a JavaScript file pretending to be a server — but actually it is a deeply processed artifact of a bundle pipeline designed for a completely different target. Configuration is unavoidable. Plugins are unavoidable. Version conflicts between plugins are unavoidable. The question is not whether your Webpack configuration will break — it will — but how many hours it will take to fix it.

Then consider the deployment monoculture. Vercel's business model is SaaS hosting for Next.js applications. Their incentive is not to make self-hosting easy — it is to make self-hosting hard enough that "just deploy to Vercel" is the obvious answer. The framework they created and maintain has deployment characteristics that are genuinely difficult to replicate outside their infrastructure. ISR, edge functions, image optimization — all of these integrate smoothly on Vercel and require significant effort everywhere else. This is not an accident.

The cultural root cause, stated plainly: frontend engineers assumed backend responsibilities without backend discipline. They built frameworks from the component tree outward rather than from the server inward. And a major framework vendor has financial incentives to perpetuate this complexity.

## What Clean Looks Like

Rails. Django. Laravel. Phoenix.

These frameworks were built by backend engineers who added a frontend layer when they needed one. They started with the server — routing, request handling, database access, business logic — and built outward to templates or client integration.

Every one of them ships with first-party solutions for the problems every web application has: authentication, routing, an ORM, migrations, test infrastructure, email, background jobs. You clone the repo, run one command, and you have a working foundation. Not a foundation that requires you to choose between twelve authentication libraries, configure a bundler, decide whether to use REST or GraphQL or tRPC, pick a CSS solution, and wire everything together.

Rails is opinionated. DHH makes decisions and is comfortable being wrong in the same direction for the entire community. The community accepts the opinions because they see the result: a framework that has been in production for twenty years, that powers GitHub and Shopify and Basecamp and thousands of companies you have never heard of, that has not required you to rewrite your authentication layer every three years because the recommended auth library changed.

The dirty secret of the JavaScript framework space is that no JavaScript framework has earned the right to have opinions the way Rails earned it. Every JavaScript framework either refuses to have opinions (here are 30 routing libraries, pick one) or has opinions that change every eighteen months (class components → function components → hooks → Server Components → Server Actions). Neither builds the trust that makes an opinionated framework work.

Forge intends to earn that trust by being correct from the start — by making design decisions that do not need to be revisited because they were made from the right foundations.

## The Browser Constraint

JavaScript is mandatory on the client. This is a permanent constraint of the web platform, not a preference, not a legacy burden that can be wished away. The browser runs JavaScript. Any web application that runs in a browser must compile to JavaScript.

The question is not whether to use JavaScript. The question is who controls the JavaScript toolchain.

The current answer is: nobody controls it. The toolchain is a negotiation between esbuild, Babel, SWC, Webpack, Rollup, Vite, PostCSS, autoprefixer, TypeScript, and whatever version constraints the installed plugins impose. Every project assembles this toolchain differently. Every project has a slightly different configuration. The output is JavaScript, but the path to that output is bespoke to each project.

The Forge answer: the framework controls the toolchain. The Rust-based compiler is not an external tool you configure — it is part of the framework. When you install Forge, you get a single binary that knows how to compile Forge applications. The JavaScript output is deterministic, reproducible, and correct by construction — not by convention.

This model is not novel. The Go compiler controls Go's toolchain. Cargo controls the Rust compilation pipeline. Elm controls how Elm compiles to JavaScript. These languages have excellent tooling stories precisely because the language authors made the deliberate choice to own the toolchain rather than delegate it to an ecosystem of loosely coupled plugins.

The browser still gets JavaScript. But that JavaScript was produced by a compiler that understood the full program, enforced the client/server boundary at compile time, and eliminated the dead code it knew would never run. Not by convention. Not by configuration. By construction.

## Forged, Not Assembled

This is the philosophy in one sentence.

The Node.js era produces builders who configure. You install a framework. You install the packages the framework tells you to install. You configure the bundler. You configure the linter. You configure the formatter. You configure the deployment pipeline. The artifact you deploy is something npm assembled at install time from 2,341 packages of unknown provenance, processed through a configuration pipeline you wrote by following a tutorial that was accurate when it was written but may have drifted since.

You did not build that. You assembled it. The assembly instructions are called `package.json`, `webpack.config.js`, `babel.config.json`, `.eslintrc.js`, `tsconfig.json`, `next.config.js`, `.env.local`, and `vercel.json`. Each one represents a negotiation between you and a tool. Each negotiation is a potential failure point.

Forge produces builders who build. You write application code. You express your data model. You write your server functions. You write your UI components. The compiler takes that application code and produces a deployment artifact. You did not configure the artifact — you described the application, and the compiler produced the artifact.

A forged artifact is something made. An assembled artifact is something configured. The difference is not just aesthetic. A made thing has structural integrity. An assembled thing has structural complexity. Structural integrity means: when something breaks, it breaks at a place you can reason about. Structural complexity means: when something breaks, it breaks at the intersection of two plugin version constraints that both changed last week.

Forge applications are made. The compiler understands the full program. The FSL provides the standard patterns. The binary is self-contained. You built it.

## The Technical Foundation Is Ready

Forge is not a research project. The technology required to build it exists and is production-proven.

**Oxc** is the fastest JavaScript and TypeScript parser available, written in Rust, built on an arena allocator that enables zero-copy AST operations. It is maintained by the VoidZero team — the same engineers who built Vite — and it is already powering Rolldown and Oxlint. The days of "Rust can parse JS but the API is too hard to use" are over.

**Rolldown** is a Rollup-compatible Rust bundler, also from VoidZero. It is the planned replacement for the esbuild bundler inside Vite. It bundles significantly faster than any JavaScript bundler and supports the module graph analysis needed for dead-code elimination and boundary enforcement.

**deno_core** is four years of production-proven work on embedding V8 with a Tokio event loop, a module loader, and an op registration system. Deno uses it in production. It is the right foundation for embedding a JavaScript engine in a Rust binary — not because it is the only option, but because it handles the hard parts (module resolution, async task scheduling, structured concurrency) correctly.

**TC39 Signals** is advancing through the standards process as a native JavaScript reactive primitive. The polyfill is production-ready today. When Signals land in V8 natively, Forge's compiled output gets the performance benefit for free. No migration needed — the standard catches up to the model Forge was already built on.

**WinterTC** (Ecma TC55) is the industry standards body for server-side JavaScript APIs. Cloudflare, Deno, Node.js, and others are members. The WinterTC API set — fetch, Request/Response, crypto, streams — is the intersection of what every modern server-side runtime supports. Targeting WinterTC means targeting every runtime.

What was missing before Forge was not the technology. The technology was there. What was missing was someone willing to make the opinionated decisions required to unify it into a coherent framework, absorb the short-term adoption costs of those decisions, and maintain the conviction that correct architecture matters more than immediate compatibility.

## The Long-Term Vision

What Cargo and `rustc` are to Rust, Forge is to JavaScript.

Cargo is not a better npm. It is what a package manager looks like when it is designed correctly from the start: content addressing, deterministic installs, a single registry with author-scoped namespaces, publishing the actual source, compile-time verification of API compatibility. You do not configure Cargo to work correctly — it works correctly by design.

`rustc` is not a configurable compilation pipeline. It is a compiler. You give it Rust code. It produces a binary. The contract is clear, the output is deterministic, and the error messages tell you what is wrong in terms of the language semantics, not in terms of which plugin configuration is incorrect.

Forge is that for JavaScript. Not a meta-framework that wraps existing tools. Not a better npm. Not a more convenient way to configure webpack. A ground-up rethink of what JavaScript tooling should look like when designed by backend engineers, for the full-stack problem, in 2026 and beyond.

The JavaScript ecosystem deserves a framework that earns the right to have opinions. Forge earns it by being built on correct foundations, by owning the full pipeline, by enforcing correctness at compile time, and by refusing to sacrifice architectural integrity for short-term adoption.

The era of assembling JavaScript applications from thousands of untrusted packages is ending. The era of forging them begins here.
