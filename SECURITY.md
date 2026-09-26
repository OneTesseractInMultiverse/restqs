# Security Policy

## Supported Versions

The latest published minor line receives vulnerability fixes.

| Version | Status    |
|---------|-----------|
| `0.1.x` | Supported |

## Reporting A Vulnerability

Report suspected vulnerabilities through [GitHub private vulnerability reporting](https://github.com/OneTesseractInMultiverse/restqs/security/advisories/new).
Sign in to GitHub, or open the repository's **Security → Advisories → Report a vulnerability** page. Private reporting
is enabled; reports are shared privately with the repository's security maintainers.

Include the affected version, impact, reproduction steps, and suggested fix if known. Do not open a public issue for a
vulnerability before a fix or disclosure plan is ready. Remove credentials and personal or production data from examples.

GitHub is the sole private reporting channel. If the form is unavailable, check [GitHub Status](https://www.githubstatus.com/)
and retry after service is restored. Do not post vulnerability details in public issues, discussions, or pull requests.
Keep follow-up details in the private report; there is no public email fallback.

## Maintainer Verification

The repository owner triages private reports. Each maintainer responsible for reports must watch the repository with
**Custom → Security alerts** (or **All Activity**) and enable delivery under
[notification settings](https://github.com/settings/notifications): **Watching** and **Participating, @mentions and custom**
should include GitHub notifications and email. These are personal settings; adding a maintainer does not configure them
automatically. Verify them when maintainership changes and before releases.

Confirm repository reporting status with the local GitHub CLI:

```sh
gh api repos/OneTesseractInMultiverse/restqs/private-vulnerability-reporting
```

Expect `{"enabled":true}`. From a signed-out session, the public
[advisories page](https://github.com/OneTesseractInMultiverse/restqs/security/advisories) must show **Report a vulnerability**;
following it must prompt for sign-in with a return path to this repository's reporting form. Do not submit a dummy
vulnerability to test the route. This checks availability and configured delivery, not receipt of an actual notification.
See [GitHub's reporting and notification setup](https://docs.github.com/en/code-security/how-tos/report-and-fix-vulnerabilities/configure-vulnerability-reporting/configure-for-a-repository).

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
