# Unsigned loopback operator console

Status: **contributor procedure; installed and stranger qualification pending**

Owner: Bullet Farm maintainers

Last reviewed: 2026-09-11

This procedure uses source from
[neverhuman/bulletfarm](https://github.com/neverhuman/bulletfarm) to start local
farmd and the Portal and log into the TUI. It grants no installation, provider
or release authority. Operating HOLD remains. See [source setup](source-setup.md)
for the blocked bootstrap path and the separate canonical family proof layout.

## Prerequisites

| Need | Pin / rule |
| --- | --- |
| OS | Linux |
| Rust | Hub 1.95.0; Kernel 1.97.1, selected by each member's `rust-toolchain.toml` |
| Node / npm | 22.23.2 / 10.9.8 |
| Just | Must parse the checked-in `[positional-arguments]` recipes; 1.51.0 supports them |
| Shell tools | Git, Bash, curl, jq, sed, GNU coreutils and util-linux (`setsid`, `flock`) |
| Clone | The aggregate with all four member source trees; a Hub-only clone is insufficient |
| State | New absolute directory under your home, mode 0700, outside the clone and `/tmp` |

Keep bootstrap tokens, session cookies and CSRF tokens private. Do not copy
another operator's session file. Public record IDs are distinct from credentials.

## Start and log in

From the aggregate root:

```bash
cd bullet-farm
just -- console --data-dir "$HOME/.local/state/bullet-operator-console"
```

Put `--` before `console`; placing it after the recipe forwards an unsupported
argument. The wrapper creates missing private state directories, builds `bullet`
and `bullet-farmd` when the explicit `BULLET_BIN` / `BULLET_FARMD_BIN` pair is
unset, runs `farm init`, installs the locked Portal dependencies with lifecycle
scripts disabled, and starts farmd and Vite. It checks the fresh Vite startup
log, page and same-origin health proxy before returning access instructions.

The defaults are farmd `http://127.0.0.1:7420` and Portal
`http://127.0.0.1:5173`. To choose free ports, pass both settings as needed:

```bash
just -- console --data-dir "$HOME/.local/state/bullet-console-alternate" \
  --bind 127.0.0.1:7421 --portal-origin http://127.0.0.1:5174
```

The launcher prints `farmd=`, `origin=`, `portal=` and the private bootstrap
file path, without printing the token. Use those exact values. With the default
source-built binaries and ports, login consumes the one-time token:

```bash
cd ../bullet-kernel
./target/debug/bullet auth login \
  --farmd http://127.0.0.1:7420 \
  --origin http://127.0.0.1:5173 \
  --stdin < "$HOME/.local/state/bullet-operator-console/custody/bootstrap.token"
./target/debug/bullet tui
```

Open the printed Portal URL. CLI login does not give the browser a cookie.
The browser needs its own valid one-time bootstrap token; **Check session** only
checks a browser that already authenticated. A consumed token cannot be reused.

## Lifecycle and visible outcomes

The current launcher **returns after startup**, leaving farmd and Vite running.
Closing that terminal or pressing Ctrl+C afterward does not provide supervised
shutdown. Both console and farmd wrapper `--stop` commands refuse with
`STOP_CUSTODY_UNAVAILABLE`: persisted PIDs cannot authorize signals. A second
start with those PID files also refuses. Preserve state and logs for process
custody reconciliation; deleting PID files is not recovery. Installed service
supervision and restart/stop qualification remain unfinished.

TUI authentication and live projections do not establish Head execution or a
successful coding transaction. Retained captures show HOLD/UNBOUND, blocked Head
Send and an unknown release decision; empty rows are not a verified fleet.
`just dev` starts HTTP development shells without provisioning login, and
`just preview` requires independent canonical member checkouts. Neither is an
aggregate installation check.

The [retained console media](../../media/operator-console/README.md) is diagnostic:
provider `none`, no Candidate, different capture sources and times. The existing
[xbabe2 work order](../assurance/xbabe2-development-closeout.md#real-1080p-capture-as-maintained-source)
defines the remaining installed workflow and real synchronized capture acceptance.
