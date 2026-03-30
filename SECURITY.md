# Security Policy

## Supported Versions

Forge is pre-alpha. Only the latest commit on `main` is supported.

| Version | Supported |
|---------|-----------|
| main    | Yes       |
| < main  | No        |

## Reporting a Vulnerability

Report security vulnerabilities by emailing **security@forgejs.com**.

Do not open a public GitHub issue for security vulnerabilities.

We will acknowledge your report within 72 hours and provide an estimated fix timeline within 7 days.

## Security Design

Forge's security model is designed from the ground up:

- **No npm supply chain**: The Foundry registry uses cryptographic author identity, eliminating name-squatting attacks
- **Compile-time boundary enforcement**: Server-only code cannot accidentally reach the client bundle
- **WinterTC API surface**: The server runtime exposes only standard web APIs — no subprocess execution, no raw filesystem access by default
- **BLAKE3 package integrity**: All packages are verified by content hash before execution
- **AGPL-3.0**: Network-deployed modifications must be shared, enabling community security review
