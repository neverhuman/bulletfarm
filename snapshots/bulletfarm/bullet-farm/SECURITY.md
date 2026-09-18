# Security

Status: **disclosure map; not a release floor**  
Last reviewed: 2026-09-10

Report vulnerabilities through GitHub private vulnerability reporting on the
repository that contains the bytes (`neverhuman/bullet-farm`,
`neverhuman/bullet-kernel`, `neverhuman/bullet-git`, `neverhuman/bullet-portal`,
or the aggregate `neverhuman/bulletfarm`).

Do not file public issues that include bootstrap tokens, session cookies, CSRF
values, provider keys, or home-directory paths.

`agent/security-policy.toml` and `ops/ci/security.sh` are CI policy for this
Hub. They are not a disclosure inbox and they do not clear G8.

License: Apache-2.0. See [`LICENSE`](LICENSE).
