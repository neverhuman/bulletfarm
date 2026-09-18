# Security

Status: **private reporting unavailable; release security acceptance incomplete**

Last checked: 2026-09-11

GitHub's read-only repository API reported private vulnerability reporting
**disabled** for the aggregate and all four members:

- [Aggregate reporting status](https://api.github.com/repos/neverhuman/bulletfarm/private-vulnerability-reporting)
- [Hub reporting status](https://api.github.com/repos/neverhuman/bullet-farm/private-vulnerability-reporting)
- [Kernel reporting status](https://api.github.com/repos/neverhuman/bullet-kernel/private-vulnerability-reporting)
- [BulletGit reporting status](https://api.github.com/repos/neverhuman/bullet-git/private-vulnerability-reporting)
- [Portal reporting status](https://api.github.com/repos/neverhuman/bullet-portal/private-vulnerability-reporting)

Until maintainers publish a verified private channel, open a
[Hub issue](https://github.com/neverhuman/bullet-farm/issues) asking only for a
preferred private security contact. Do not include the vulnerability, exploit,
affected secrets or other sensitive details in that public request. This follows
[GitHub's reporting guidance](https://docs.github.com/en/code-security/how-tos/report-and-fix-vulnerabilities/report-privately).

Private reporting must be enabled and checked from a public visitor's perspective
before this policy directs people to submit reports through it. Adding this file
does not enable the feature. No security mailbox or response deadline is promised
by this alpha.

Keep bootstrap tokens, session cookies, CSRF values, provider keys and private
configuration out of public issues, pull requests and diagnostic attachments.
Review any diagnostic export before sharing it.

There is no certified release or supported-version security matrix yet. The
existing [product gap register](docs/assurance/product-gaps.md) records security
acceptance work. `agent/security-policy.toml` and `ops/ci/security.sh` are Hub CI
policy, not a disclosure inbox or evidence that those obligations have passed.

Members carry Apache-2.0 notices; see [LICENSE](LICENSE). Separately licensed
third-party material retains its own terms.
