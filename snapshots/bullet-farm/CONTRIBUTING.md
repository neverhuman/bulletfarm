# Contributing

Status: **source contribution guidance; installed release remains blocked**

Last reviewed: 2026-09-11

The public entry is [neverhuman/bulletfarm](https://github.com/neverhuman/bulletfarm),
one aggregate checkout containing four member source trees. Source changes belong
in the corresponding member repository:

| Files in the aggregate | Pull request destination |
| --- | --- |
| `bullet-farm/` | [Hub](https://github.com/neverhuman/bullet-farm) |
| `bullet-kernel/` | [Kernel](https://github.com/neverhuman/bullet-kernel) |
| `bullet-git/` | [BulletGit](https://github.com/neverhuman/bullet-git) |
| `bullet-portal/` | [Portal](https://github.com/neverhuman/bullet-portal) |

Root aggregate files are generated from the Hub's `publication/root/` templates
and `publication/config.json`. Propose those changes in the Hub. Reviewed member
commits are then captured into an aggregate publication; direct aggregate edits
fail regeneration checks. Independent review and approval of publication remain
separate from local source work.

## Source builds and local console

Follow [source setup](docs/runbooks/source-setup.md) for the aggregate build
commands and the different canonical family layout. Canonical proof requires
four independent member checkouts and their admitted tool/source subjects; an
aggregate subtree, source archive or shallow history does not provide that
custody. Do not create Git worktrees or set CI variables to bypass admission.

The unsigned console requires Linux, both checked-in Rust toolchains
(Hub 1.95.0 and Kernel 1.97.1), Node 22.23.2, npm 10.9.8 and
Just with support for the checked-in `[positional-arguments]` recipes
(1.51.0 supports them), plus Git, Bash, curl, jq, sed, GNU coreutils and
util-linux (setsid and flock). It starts local farmd and a Vite Portal in the selected state directory.
For a new local instance, choose a new directory. From `bullet-farm/`:

```bash
just -- console --data-dir "$HOME/.local/state/bullet-operator-console"
```

Place `--` before the recipe name; putting it after `console` forwards an
unsupported argument. Select a state directory under your home, outside the
clone and `/tmp`. The launcher returns after startup, leaving farmd and Vite
running. Its access instructions identify their origins and the private
bootstrap file; never paste that token into a public report. Closing the launcher
terminal or pressing Ctrl+C afterward does not provide supervised shutdown.
A later `--stop` invocation refuses because persisted PIDs do not grant process
custody. Preserve state and logs for reconciliation; deleting PID files is not
recovery. See the [loopback guide](docs/runbooks/loopback-console.md) for login
and the current lifecycle limits.

`just setup` still requires an admitted external bootstrap and explicit tool
subjects, then refuses the checked-in schema-2 lock. `just preview` is for the
canonical family and expects a BLOCKED diagnosis. Neither command establishes
installed service, provider execution, or release certification. Operating HOLD
remains in effect.

## Verification and review

Keep changes small and explain the behavior, exact source commit, focused checks,
and remaining limitations in the pull request. Follow the owning member's local
instructions and mapped checks; preserve failures as well as successful results.
A skipped, unavailable or zero-test result cannot stand in for required execution.
Canonical required lanes need their own source and tool admission.

Changes to provider enrollment, signing, forge effects and release publication
retain their separate approval requirements. Opening a source pull request does
not grant those effects. Keep credentials and private diagnostic contents out of
issues and pull requests; use [security reporting](SECURITY.md) for vulnerabilities.

Members carry Apache-2.0 notices; see [LICENSE](LICENSE). Preserve third-party
licenses and attribution when changing or redistributing their files.
