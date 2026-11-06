# Amendment A02 — Dev container is the only place code is built and tested

**Date:** 2026-11-06 · **Approved by:** Jerry · **Amends:** AGENTS.md, A01, tasks/C0, tasks/S1
**Adds:** task S0 (done — records the dev container that exists)

Read after A01. Where this amendment disagrees with earlier files, this amendment wins.

## 1. The dev container (as built, task S0)

| Item | Value |
| --- | --- |
| Image | `vstone-dev:latest`, built from `Dockerfile.dev` (`python:3.12-slim-bookworm` + rustup) |
| Rust | pinned in `rust-toolchain.toml`; `Dockerfile.dev` installs the same version — bump both together |
| Container | `Vorona-dev`, long-lived (`make dev-up`), repo mounted at `/src` |
| Volumes | `vstone-cargo-target` (→ `/src/core/target`), `vstone-cargo-registry` |
| Limits | `--cpus=4 --memory=4g` |
| Commands | `make dev-image`, `make dev-up`, `make dev-check`, `make dev-shell`, `make dev-down` |

`make dev-check` (= `pip install --no-deps -e .` + `make check` inside `Vorona-dev`) is the check that counts.
`make check` on the host does not count. A01's `docker-check` becomes `dev-check` until S8 lands.

## 2. Rules (all agents)

1. **Only container results count.** A result counts only if `make dev-check` produced it in
   `Vorona-dev` on the MacBook and the raw output is pasted (or saved to `runs/logs/` and linked).
   Results a subagent reports from any other environment do not count.
2. **Subagents write code; they do not certify it.** The agent with a shell on the MacBook runs the checks.
3. **Prove the machine.** Each session's first report starts with the raw output of
   `uname -a; docker ps --format '{{.Names}}'`.
4. **Leave other Docker things alone.** Jerry runs Home Assistant in Docker on this Mac.
   Only create, stop or remove containers, images and volumes named `Vorona-*` or `vstone-*`.
   Never run any `docker * prune`, never touch other containers. Docker Desktop settings may be
   changed only when a task needs it, and the change must be written in the card's Agent notes.
5. **No Rust on the host.** No `rustup`/`cargo` installs on the Mac (AGENTS.md §7a).
   Host build artifacts (`core/target/` on the Mac, any `.dylib`) are not evidence of anything.
6. **Subagent workspace.** The `rust-kernel` subagent must work in
   `~/Projects/Vorona/voronoi-stone`, not in another project, and must not copy conventions from
   other projects unless a card asks for it.

## 3. Task S0 — Dev container

**Owner:** SHELL · **Status:** done 2026-11-06 (commits 05b361a, 04eb04c, 831406b, 6483e74).
Acceptance: `make dev-image` builds; `make dev-check` runs fmt, clippy, cargo test, ruff and pytest
inside `Vorona-dev`; Home Assistant untouched.
