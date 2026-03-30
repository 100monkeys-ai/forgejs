# ADR-004: AGPL-3.0-Only License

**Number**: 004
**Title**: AGPL-3.0-Only License
**Date**: 2026-03-30
**Status**: Implemented
**Deciders**: Jeshua (Architect)
**Tags**: `#license` `#legal` `#open-source` `#governance`

---

## Context

Forge is being built as part of the 100monkeys platform, which is licensed under the GNU Affero General Public License version 3.0 (AGPL-3.0). The decision of which license to apply to Forge requires evaluating the project's goals, its relationship to the broader 100monkeys ecosystem, and the precedents set by comparable open-source infrastructure projects.

The core tension in choosing an open-source license for a developer framework is adoption versus protection:

**The adoption argument for permissive licenses (MIT, Apache-2.0)**: A permissive license removes all friction for commercial adoption. Companies can build proprietary products using MIT-licensed frameworks without any obligation to contribute back or disclose their usage. React, Vue, and most of the JavaScript framework ecosystem are MIT-licensed. The low friction of MIT licenses has contributed to their dominant adoption in the JavaScript space. Forge is a new framework competing against established players — permissive licensing would minimize the barrier to evaluation.

**The protection argument for copyleft licenses (GPL, AGPL)**: A copyleft license ensures that improvements to the framework are shared back with the community. For a project maintained by a small team, copyleft prevents the "Amazon problem" — a well-resourced company takes the open-source project, hosts it as a managed service, and extracts value from the community's labor without contributing back. PostgreSQL, Linux, and many foundational infrastructure projects use copyleft licenses for this reason.

The specific question is not just "copyleft or not" but "which copyleft." GPL and AGPL differ in a crucial way for web frameworks.

**GPL's distribution trigger**: The GPL requires that derivative works be distributed under GPL terms when they are *distributed* — i.e., when the compiled binary is given to another party. A web application running on a server is never distributed to end users; the HTML/CSS/JS that reaches users is the application's output, not the framework itself. A company can run a GPL-licensed web framework on their servers, build a proprietary application on top of it, and never trigger the GPL's copyleft requirement.

**AGPL's network use trigger**: The AGPL extends the GPL's copyleft to cover *network use*. Under AGPL, if you run a modified version of an AGPL-licensed program as a network service — i.e., if users interact with it over a network — you must make the source of your modifications available. For a server-side framework, this is the right clause: the relevant "distribution" event for a web framework is running it as a service, not shipping a binary.

The AGPL model has been validated by successful commercial open-source projects at scale: MongoDB (SSPL, which is AGPL-derived), Grafana, Gitea, Mastodon, and others. The pattern works — open source with copyleft protection for network services is a viable model for commercial sustainability.

## Decision

Forge is licensed under the GNU Affero General Public License version 3.0 (AGPL-3.0-only).

"AGPL-3.0-only" rather than "AGPL-3.0-or-later" means that Forge is licensed under exactly AGPL-3.0, and contributors must accept that their contributions are licensed under AGPL-3.0. Future versions of the AGPL would require a deliberate decision to relicense.

This decision is consistent with the rest of the 100monkeys platform and does not require maintaining dual license structures.

## What AGPL-3.0 Means for Forge Users

**Building applications with Forge**: A company building a web application using Forge does not need to open-source their application code. AGPL copyleft applies to the *framework* (Forge itself), not to the work you build with it. Your route handlers, your data models, your business logic — these are your proprietary code. The AGPL does not reach them.

**Modifying Forge and deploying the modification**: If a company modifies the Forge framework code — the compiler, the runtime, the FSL, the bundler — and runs that modified version as a network service, the AGPL requires them to make the source of those modifications available to users of the service. They do not need to publish it on a public repository; they must make it available on request to the service's users.

**Hosting Forge as a managed service**: If a cloud provider offers "managed Forge hosting" — a service where customers deploy Forge applications to the provider's infrastructure — and if that service involves modifications to Forge itself, those modifications must be open-sourced. This is the key protection against the "Amazon problem."

**Commercial licensing for AGPL exceptions**: Companies that cannot comply with the AGPL (because they need to modify Forge without disclosing modifications) can obtain a commercial license from 100monkeys. This dual-licensing model is the standard commercial open-source pattern used by MySQL, Qt, and others.

## Consequences

### Positive

- ✅ **Consistent with the platform**: the 100monkeys platform is AGPL-3.0. A different license for Forge would create legal complexity at the integration points between Forge and the rest of the platform.
- ✅ **Network copyleft is the right model for a server framework**: the AGPL's trigger (network use) is precisely the trigger that matters for a framework that runs web servers. GPL's distribution trigger would not fire in the primary use case.
- ✅ **Prevents the hosted service fork problem**: Forge's network copyleft ensures that if a major cloud provider takes Forge, modifies it, and hosts it as a service, the modifications must be shared. This is the primary risk for a small team maintaining infrastructure software.
- ✅ **Commercial licensing path is viable**: the AGPL-3.0 + commercial exception model (offer a paid commercial license for users who cannot comply with AGPL) is a validated revenue model. It does not require Forge to be proprietary to be commercially sustainable.
- ✅ **Contributor agreements are straightforward**: all contributors license their contributions under AGPL-3.0. No contributor license agreement (CLA) is required beyond the standard "your code, once merged, is AGPL-3.0."

### Negative

- ❌ **Some companies have blanket AGPL policies**: several large technology companies (including some that significantly contribute to the JavaScript ecosystem) have policies that prohibit using AGPL-licensed software in their products. These companies will not evaluate Forge as an option regardless of the technical merit. This is a real adoption cost.
- ❌ **AGPL compliance is not always well-understood**: some developers and their legal teams conflate "AGPL-licensed" with "cannot be used in commercial software." This is incorrect — AGPL applies to modifications of Forge itself, not to applications built with Forge — but the misunderstanding creates friction.
- ❌ **May slow initial adoption compared to MIT**: in a head-to-head comparison where Forge and a competitor are technically equivalent, a developer choosing for a commercial project may default to the MIT-licensed option to avoid legal review. This is a real but bounded risk — Forge's technical differentiation must be large enough to be worth the license discussion.

### Neutral

- ℹ️ AGPL-3.0 is OSI-approved and FSF-endorsed. It is a legitimate open-source license, not a "source available" proprietary license in open-source clothing.
- ℹ️ The FSL packages (forge:auth, forge:data, etc.) are part of the Forge monorepo and therefore AGPL-3.0. They are not separately licensed as MIT.
- ℹ️ AGPL-3.0 is compatible with GPL-3.0 libraries (GPL-3.0 code can be incorporated into an AGPL-3.0 project). AGPL-3.0 is compatible with MIT, Apache-2.0, and most permissive licenses.

## Alternatives Considered

### MIT

Maximum adoption, no copyleft. The JavaScript framework ecosystem norm. Choosing MIT would mean:

1. No protection against a cloud provider hosting a proprietary "managed Forge" fork
2. No mechanism to ensure improvements to the compiler or FSL are shared back
3. Commercial sustainability requires a different model (support contracts, hosted SaaS built on Forge)

The MIT option is not wrong — it is a coherent choice that many successful projects make. It is wrong for Forge at this stage because the 100monkeys team is small, the framework is technically ambitious, and the most likely path to it becoming a footnote is a well-resourced company taking it, hosting it, and not contributing improvements back. AGPL-3.0 is the correct instrument to prevent this.

### Apache-2.0

Similar to MIT in the adoption/protection tradeoff. Apache-2.0 adds an explicit patent grant and a "Contributor License Agreement" substitute clause. For a framework, the patent provisions matter less than the copyleft question — Apache-2.0 provides no copyleft, so the same "Amazon problem" risk applies.

Rejected for the same reasons as MIT.

### GPL-3.0

GPL-3.0 provides copyleft for distributed software. For a server-side framework, the distribution trigger almost never fires — web applications run on servers, and the server never distributes the framework binary to end users.

A company could take Forge, modify it, deploy it as a service, and have no GPL obligation, because they are not distributing the binary. This is the exact gap the AGPL was created to close. GPL-3.0 without the AGPL's network use clause provides insufficient protection for a server framework.

Rejected: the distribution trigger is the wrong trigger for a server-side framework.

### BSL (Business Source License)

The BSL (used by CockroachDB, Sentry, and others) makes code "source available" under a non-open-source license for a time period, then converts to an open-source license (usually Apache-2.0) after a specified date (typically 4 years).

The BSL is not an open-source license by OSI definition. The "available source" marketing around BSL projects creates confusion, and some developers and companies have strong objections to it.

More importantly, the 100monkeys platform is AGPL-3.0, and the principle of that choice is genuine open-source with copyleft — not "source visible until we feel safe." The BSL model is inconsistent with the platform's philosophy.

Rejected: not consistent with the project's open-source principles.

### MIT + Commercial Exception (the "Commons Clause")

The Commons Clause is an addendum to permissive licenses that prohibits selling the software as a hosted service. MIT + Commons Clause has been used by Redis Labs and Confluent.

The Commons Clause is also not an OSI-approved open-source license. It was controversial in the open-source community when introduced. The AGPL-3.0 achieves the same protection goal (preventing hosted service forks) through a mechanism that is unambiguously open-source and well-understood by the legal community.

Rejected: AGPL-3.0 is a cleaner, more legally established instrument for the same protection goal.

## Related Decisions

This decision is consistent with and derives from the licensing philosophy of the 100monkeys platform (AGPL-3.0 per platform CLAUDE.md).
