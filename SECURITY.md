# Security Policy

## Supported Versions

The latest published minor line receives vulnerability fixes.

| Version | Status    |
|---------|-----------|
| `0.1.x` | Supported |

## Reporting A Vulnerability

Report suspected vulnerabilities through GitHub private vulnerability reporting:

https://github.com/OneTesseractInMultiverse/restqs/security/advisories/new

Include the affected version, impact, reproduction steps, and suggested fix if known. Do not open a public issue for a
vulnerability before a fix or disclosure plan is ready.

## Security Expectations

RestQS parses untrusted query strings. Changes must not turn raw field names into SQL identifiers. Changes must not
concatenate user values into SQL text.

Keep regex disabled by default. Keep text search unsupported until a dialect-specific adapter contract exists.

Security review focuses on these areas:

- Identifier allowlists.
- Value binding.
- Parser limits.
- Regex gates.
- Error text and log safety.
- Feature-gated adapters.

See `docs/security-model.md` for the full security model.
