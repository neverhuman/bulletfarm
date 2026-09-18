# Getting help with Bullet

Start with [source setup](bullet-farm/docs/runbooks/source-setup.md) and the
[contribution guide](CONTRIBUTING.md). The public aggregate contains four member
source trees; they are not four independent Git checkouts.

For setup questions, an unclear error, or a problem spanning members, open a
[Hub issue](https://github.com/neverhuman/bullet-farm/issues). For a bug with a
known owner, use the corresponding member repository:

| Area | Issue tracker |
| --- | --- |
| CLI, TUI, scheduling, or Runner | [Kernel](https://github.com/neverhuman/bullet-kernel/issues) |
| Candidate, workspace, or Git journal | [BulletGit](https://github.com/neverhuman/bullet-git/issues) |
| Browser interface | [Portal](https://github.com/neverhuman/bullet-portal/issues) |
| Installation, documentation, or aggregate generation | [Hub](https://github.com/neverhuman/bullet-farm/issues) |

Include the aggregate or member commit, operating system and architecture, the
command or action, expected behavior, observed behavior, and a small reproduction.
For a failed check, include its exit status and which tests actually ran. Say when
an operation's outcome is unknown; preserve its original request identity before
retrying. Review and redact logs before sharing them. Do not attach credentials,
private state directories, provider transcripts, or repository contents without
reviewing what they contain.

For a suspected vulnerability, follow [Security](SECURITY.md). Public issues are
not a private disclosure channel.

Bullet is under development. There is no certified release, support service, or
promised response time. Current limitations are recorded in the existing
[product gap register](bullet-farm/docs/assurance/product-gaps.md).
