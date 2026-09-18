# Bullet Farm

[neverhuman/bulletfarm](https://github.com/neverhuman/bulletfarm) is the public
aggregate Git repository. It contains four member source trees:

| Member | Role |
| --- | --- |
| [bullet-farm](bullet-farm/README.md) | Hub, installer, contracts, and assurance |
| [bullet-kernel](bullet-kernel/README.md) | Control plane and runtime boundaries |
| [bullet-git](bullet-git/README.md) | Candidate graph, journal, and proof roots |
| [bullet-portal](bullet-portal/README.md) | Operations portal |

Generated snapshots record each member's commit, tree, object format and source
ref in [`publication.json`](publication.json). Original source objects are retained
under `refs/tags/bullet-source/v1/<member>/<commit>`. An aggregate clone has one
Git checkout; its member directories are source trees, not independent checkouts.

Bullet is not release certified. The existing
[G1–G18 register](bullet-farm/docs/assurance/product-gaps.md) and
[full-product plan](bullet-farm/docs/assurance/full-product-dogfood-plan.md)
track the remaining work. Generated workflows derive jobs and matrix cells from
the reviewed member catalog; unavailable execution remains non-passing.
Installed operation, authenticated provider tasks, independent verification and
approved integration require their own evidence. Operating HOLD remains.

Public installation is not available. The checked-in [family.lock](bullet-farm/family.lock) is still schema 2, which the installer refuses. See the [source setup runbook](bullet-farm/docs/runbooks/source-setup.md) for the admitted bootstrap requirements.

## Clone and build from source

The current Linux source build provides an authenticated CLI/TUI and a local
Portal. `bullet` and `bulletfarm` open the same TUI by default. Durable Head
conversation, supervised provider terminals and an installed daily-coding
service remain incomplete. Operating HOLD remains; the instructions below
start an unsigned contributor console and do not enroll a provider account.

### Prerequisites and exact build

| Tool | Required source pin or prerequisite |
| --- | --- |
| Rust for Kernel and its CLI | 1.97.1, selected by `bullet-kernel/rust-toolchain.toml` |
| Rust for Hub checks | 1.95.0, selected by `bullet-farm/rust-toolchain.toml` |
| Node / npm | 22.23.2 / 10.9.8; the console checks both |
| Just | Support for `[positional-arguments]`; 1.51.0 supports these recipes |
| Linux tools | Bash, Git, curl, jq, GNU coreutils, sed/grep and util-linux (`setsid`, `flock`) |

Install the selected tools first and make them available on `PATH`. Use one
Bash session for the following commands; quoted paths also support spaces.
From a new aggregate checkout:

```bash
git clone https://github.com/neverhuman/bulletfarm.git bulletfarm
cd bulletfarm
BULLET_SOURCE_ROOT="$PWD"
BULLET_BUILD_ROOT="$HOME/.cache/bullet-source-build"
git rev-parse HEAD

(cd "$BULLET_SOURCE_ROOT/bullet-kernel" && \
  CARGO_TARGET_DIR="$BULLET_BUILD_ROOT" cargo build --locked \
    -p bullet --bin bullet --bin bulletfarm \
    -p bullet-farmd --bin bullet-farmd)

BULLET_CLI="$BULLET_BUILD_ROOT/debug/bullet"
BULLET_DAEMON="$BULLET_BUILD_ROOT/debug/bullet-farmd"
"$BULLET_CLI" --help
sha256sum "$BULLET_CLI" "$BULLET_BUILD_ROOT/debug/bulletfarm" "$BULLET_DAEMON"
```

Keep the source commit and binary hashes with diagnostics. These are local
build identities, not signed release evidence. The executable is at the path
above; this build does not add `bullet` to your `PATH`. The sibling `bulletfarm`
binary has the same command parser, state and exit behavior.

The aggregate has one Git checkout. Its four member directories are source
trees; canonical family proof and publication use independent member checkouts.
`just preview` is for that canonical layout. `just setup` requires an admitted
external `bullet-family` and explicit Cargo/Node/npm subjects, then currently
refuses the checked-in schema-2 lock with `UNSUPPORTED_SCHEMA`. It does not
install a signed service. See [source setup](bullet-farm/docs/runbooks/source-setup.md).

### Start a fresh local instance

Choose short, private directories directly under your home, outside the clone
and `/tmp`. The server's lease socket path must remain shorter than 100 bytes.
These examples intentionally refuse if either state directory already exists:

```bash
umask 077
BULLET_SERVER_STATE="$HOME/.bullet-source-console"
BULLET_CLIENT_STATE="$HOME/.bullet-source-client"
mkdir -m 700 -- "$BULLET_SERVER_STATE" "$BULLET_CLIENT_STATE" &&

(cd "$BULLET_SOURCE_ROOT/bullet-farm" && \
  just -- console --data-dir "$BULLET_SERVER_STATE" \
    --bullet "$BULLET_CLI" --farmd "$BULLET_DAEMON" \
    --bind 127.0.0.1:7420 --portal-origin http://127.0.0.1:5173)
```

Put Just's `--` before `console`. Explicit binary paths prevent an inherited
Cargo target override from selecting the wrong build. The launcher initializes
private server state, runs the Portal preinstall scan, installs locked npm
dependencies with lifecycle scripts disabled, and starts farmd plus Vite.
It checks its own Vite startup and the proxied `/health` response before printing
`farmd=`, `origin=`, `portal=`, `bootstrap_file=` and log paths. Health means the
HTTP service is responding; it does not establish task or provider readiness.

The launcher returns with farmd and Vite running. It does not install a service,
qualify reboot recovery or provide a later supervised stop. Both console
`--stop` and the underlying dogfood stop helper currently refuse. A saved PID
file cannot authorize signaling. Read the lifecycle section before restarting.

### Authenticate and open the CLI

For the CLI, consume the new instance's token through stdin, never a command-line
argument. Keep the endpoint and Origin exactly as configured above:

```bash
"$BULLET_CLI" auth login --state-dir "$BULLET_CLIENT_STATE" \
  --farmd http://127.0.0.1:7420 --origin http://127.0.0.1:5173 \
  --stdin < "$BULLET_SERVER_STATE/custody/bootstrap.token"
"$BULLET_CLI" auth status --state-dir "$BULLET_CLIENT_STATE"
"$BULLET_CLI" tui --state-dir "$BULLET_CLIENT_STATE"
```

A successful `auth status` reports the authenticated operator, exact session and
expiry. Pass the same `--state-dir` on subsequent commands. Without that flag,
credentials default to `$XDG_STATE_HOME/bullet/operator`, or
`$HOME/.local/state/bullet/operator` when `XDG_STATE_HOME` is absent; plain
`bullet` and `bulletfarm` use that default location. A second login does not
silently replace existing credentials or resolve a pending credential record.

For the browser, open <http://127.0.0.1:5173>, enter an **unused** token in
**Operator session → One-time bootstrap token**, and select **Authenticate
local session**. **Check session** validates a browser session already created.
CLI login does not create a browser cookie, and its consumed token cannot be
reused. Choose which client consumes the new token; using both against one
instance requires separately provisioned valid sessions. The contributor
launcher does not yet document that additional-session provisioning flow.
Never copy the CLI cookie/CSRF store into the browser as a workaround.

### Use xbabe2 through SSH

Build, start and run the CLI on xbabe2 under the owning user, in an ordinary SSH
terminal. In a separate terminal on your local computer, forward the Portal
port; replace `YOUR_SSH_USER` with that account's SSH login:

```bash
ssh -N -L 127.0.0.1:5173:127.0.0.1:5173 YOUR_SSH_USER@xbabe2
```

Open the same `http://127.0.0.1:5173` URL locally. Browser API requests use Vite's
same-origin farmd proxy; do not expose farmd or Vite on a public bind address.
For an alternate instance, use unused server ports such as `7421` and `5174`,
set `--bind 127.0.0.1:7421 --portal-origin http://127.0.0.1:5174`, and use that
exact Origin in login and both `5174` positions in the SSH forward. Keep each
instance's server/client state separate. Closing the forward disconnects the
browser, but is not a daemon shutdown or a native-session lifecycle proof.

### Current commands and TUI controls

These commands observe the authenticated instance without submitting work:

```bash
"$BULLET_CLI" tui --state-dir "$BULLET_CLIENT_STATE" --once
"$BULLET_CLI" mission list --state-dir "$BULLET_CLIENT_STATE" --json
"$BULLET_CLI" coding list --state-dir "$BULLET_CLIENT_STATE" --limit 20 --json
"$BULLET_CLI" coding board --state-dir "$BULLET_CLIENT_STATE" --json
```

`--once` prints a plain TUI snapshot; it is not JSON. `coding list` pages are
bounded to 100 rows and accept `--after`. Use exact IDs returned by discovery
for `coding status <command-id>`, `coding task <command-id>` or
`tui --subject <subject-id>`, with the same client state directory. Diagnostics
go to stderr; use `--json` on commands that expose it for machine-readable data.
Piping text into the default CLI does not submit a goal to Head.

| Key in the current TUI | Action |
| --- | --- |
| Ctrl+K | Open command palette |
| Tab / Shift+Tab | Switch list/detail focus |
| Up / Down or k / j | Move selection |
| Enter / Escape | Open / go back |
| `?` / uppercase `J` | Help / raw JSON view |
| `r` | Request refresh when no read is already pending |
| Ctrl+C | Detach from Bullet's TUI; preserve server work |

The TUI first renders before credential discovery and refreshes periodically.
On detach it prints a reconnect command preserving the selected subject and
state directory. Copy that command exactly. This is Bullet's ordinary TUI;
provider-native input, controller takeover and `Ctrl+]` then `d` detachment
remain part of the unqualified native-terminal work.

`coding submit`, `retry`, `status`, `task`, `board` and `watch` exist; consult the
specific subcommand's `--help` for inputs. Submission persistence and its
command ID do not establish Head progress, a launched provider, a Candidate,
verification or integration. A lost acknowledgement requires reconciliation of
the original command in the same state directory; do not mint another command
or delete its journal to make an ambiguous result disappear. `coding stop`
currently reports `STOP_UNIMPLEMENTED`. Generic signed-in provider execution
still refuses unsupported containment; account credentials do not remove it.

### State, lifecycle and troubleshooting

Keep server and client directories private and preserve their contents:

| Location | Purpose |
| --- | --- |
| Server `ledger.sqlite` and companion files | Durable local ledger; preserve as one database state |
| Server `custody/` | Lease transport, local peer registry and bootstrap material; secret-bearing |
| Server `socket/`, PID files and `logs/` | Local socket, diagnostic process records and startup/runtime logs |
| Client state directory | Credentials and exact command journals; secret-bearing recovery state |

Do not delete PID files to bypass `OPERATOR_CONSOLE_PID_STATE_UNRECONCILED`,
use PID numbers as authority to kill processes, or copy a live SQLite file as a
qualified backup. Preserve the state and launch evidence for operator
reconciliation. A fresh directory creates a different instance; it is not a
restart, restore, upgrade or recovery of the old one. Installed lifecycle and
backup/restore qualification remain open.

| Symptom | Next check |
| --- | --- |
| Tool or build refusal | Check the pinned versions, selected member directory and explicit binary paths; retain stderr |
| No ready Portal or occupied port | Inspect the printed startup logs and port settings; never treat an unrelated listener as readiness |
| Unsafe state/ancestor refusal | Inspect ownership, permissions and symlinks for the selected path; do not broaden permissions or recursively change unrelated homes |
| Authentication missing or expired | Run `auth status` with the original client directory; preserve pending records and use a valid session |
| Browser unauthorized after CLI login | The browser needs its own bootstrap exchange; **Check session** does not log in |
| Empty views, `UNKNOWN` or blocked work | Retain the exact subject/reason and source version; empty or saved records are not completed execution |

`auth revoke --state-dir ...` requests server-side revocation and then clears
local credentials after acknowledgement. `auth forget --state-dir ...` only
forgets local credentials; it does not revoke server authority. Neither is a
work-cancellation command. Keep unresolved response-loss evidence and journals.
Before sharing diagnostics, review logs, environment and state for secrets;
never upload bootstrap tokens, cookies, CSRF material or whole credential stores.
See the [loopback runbook](bullet-farm/docs/runbooks/loopback-console.md) and
[security guidance](bullet-farm/SECURITY.md).

### Path to daily coding

`bullet chat`, `setup`, `serve`, `agent`, `sessions` and `attach` are planned
entry points and are not present in this CLI. The next usable coding milestone
requires durable Head/task admission, contained provider custody, Candidate
preservation, useful independent gates and approved integration on an installed
service. The [xbabe2 work order](bullet-farm/docs/assurance/xbabe2-development-closeout.md)
separates the first admitted three-provider task from the twelve-task campaign,
four-provider showcase, seven-day observation and complete product acceptance.
Until those predicates pass, the console is useful for development and
inspection, not a claim that real daily coding or the requested GIFs are ready.

## Contributions, security and licensing

The delivery target for reviewed changes is a pull request to
[neverhuman/bulletfarm](https://github.com/neverhuman/bulletfarm). Development
still preserves each canonical member's source identity. Root files originate
in the Hub's `publication/root/` templates and `publication/config.json`; the
reviewed publication flow assembles those templates and exact member trees
before the aggregate PR. A direct edit to generated output does not satisfy
regeneration checks. Public contributor automation for this complete flow is
still incomplete; see [Contributing](bullet-farm/CONTRIBUTING.md) and the
[publication runbook](bullet-farm/docs/runbooks/publication.md). Member review,
applicable CI, protected integration and publication approval remain required.

Follow the [security reporting guidance](bullet-farm/SECURITY.md) before sharing
vulnerability details. Private reporting was disabled on all five repositories
when checked on 2026-09-11; request a private contact without public details.

For setup questions or bug reports, see [Support](SUPPORT.md). The issue forms
ask for a reproduction and the exact source or installed version.

The aggregate uses [Apache-2.0](LICENSE); retain the member notices and
[third-party notices](NOTICE), including the Kernel qualification assets' MIT
and OFL licenses. Public-release acceptance remains pending.

## Recording evidence

Retained operator-console diagnostics (local loopback observation;
not installed or provider qualification; Operating HOLD remains):

- [operator TUI](bullet-farm/media/operator-console/operator-tui.gif)
- [operator Portal](bullet-farm/media/operator-console/operator-portal.gif)
- [captions](bullet-farm/media/operator-console/README.md)

Historical recordings remain available in the source snapshot and are not the
current operator showcase. The three retained bridge GIFs do not meet the
required native capture geometry.

- [Contained Claude Candidate recording](bullet-farm/media/dogfood/candidate/dogfood-candidate-hi.gif)
  and [reproduction record](bullet-farm/media/dogfood/README.md): a retained
  component bridge with a preserved Candidate and cleanup refusal. It does not
  establish installed Bullet authentication, TUI/Portal control of the same
  task, independent verification, protected integration or authoritative read-back.
- [Earlier vendor-CLI and Portal recordings](bullet-farm/docs/demo-gif/README.md):
  vendor explanation sessions and a Portal form demonstration, with their
  original limitations. They do not establish the requested provider workflow.
The installed TUI and packaged Portal must still demonstrate the same real
task/run/Attempt/Candidate/review/integration identities, with authenticated
provider execution and preserved original timing. Follow the existing
[xbabe2 capture specification](bullet-farm/docs/assurance/xbabe2-development-closeout.md).
No three-provider or four-provider milestone is claimed here.
