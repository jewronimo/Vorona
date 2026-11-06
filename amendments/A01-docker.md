# Amendment A01 — Docker container

**Date:** 2026-11-05 · **Approved by:** Jerry · **Amends:** AGENTS.md, PLAN.md, docs/SPEC.md,
tasks/I1-integration.md · **Adds:** task S8

All agents read amendments in `amendments/` in number order, right after AGENTS.md.
Where an amendment and an earlier file disagree, the amendment wins. Earlier files are not edited.

---

## 1. Changes to existing files

### docs/SPEC.md — new requirement
- **R-SC-5** The tool runs in a Docker container on Jerry's MacBook (`linux/arm64`, Docker Desktop).
  The same image builds, tests and runs the tool.

### AGENTS.md
- **§3 Determinism — reference platform.** The `dev`/runtime Docker image is the reference platform.
  Golden hashes are recorded and checked only inside it. A native macOS build may differ in the
  last bits and must never update a golden file.
- **§5 How to work a task, step 5.** Once S8 has landed, run `make docker-check` instead of
  `make check`. The container result is the one that counts.
- **§7 Commands — added:**
  ```
  make docker-build                          # build images (after S8)
  make docker-check                          # make check inside the dev container
  make docker-run CONFIG=configs/<name>.toml # run in the container, output to ./work/runs
  ```

### PLAN.md
- **Stack — added:** Runtime is Docker Desktop on the MacBook, `linux/arm64` image (S8) for building,
  testing and running. Ollama/Qwen runs natively on the Mac, outside Docker (Docker on macOS has no
  GPU access).
- **Two tracks — SHELL row 8:** S8 Docker image (build, test, run).

### tasks/I1-integration.md
- **Depends on:** C7, S7, **S8**.
- **Acceptance replaced:**
  - [ ] `make docker-check` + e2e tests green inside the container on Jerry's MacBook.
  - [ ] Full cylinder run ≤ 2 minutes inside the container.
  - [ ] Golden hashes recorded inside the container only.

---

## 2. New task card

# S8 — Docker image (build, test and run in one container)

**Owner:** SHELL · **Depends on:** S1 · **Unblocks:** I1
**Read first:** AGENTS.md §3 (determinism), §7 (commands), this amendment §1

## Goal
One image that builds the Rust core, runs every test, and runs `vstone`, on Jerry's MacBook
(Docker Desktop, Apple Silicon → `linux/arm64`; Intel Mac → `linux/amd64`).

## Platform facts (checked 2026-11-05)
- `build123d 0.13.0` needs `cadquery-ocp-novtk >=8.0,<8.1`; version `8.0.1.1.0` ships wheels for
  `manylinux_2_28_aarch64` and `manylinux_2_28_x86_64`, Python 3.11–3.14. So `linux/arm64` works natively.
- Use Python 3.12 and a glibc base (Debian bookworm). Not Alpine (no manylinux wheels).
- Docker on macOS has no GPU access: Ollama stays native on the Mac.

## Deliverables
- `Dockerfile`, multi-stage:
  1. `builder` — `rust:<pinned>-bookworm` + Python 3.12 + `maturin`; builds the `vstone_core` wheel
     (`maturin build --release --features python`) and the `vstone-core` binary.
  2. `runtime` — `python:3.12-slim-bookworm`; installs the wheel, the `vstone` package and pinned
     Python deps (`requirements.lock`); copies `vstone-core`; non-root user `vstone`; `WORKDIR /work`;
     `ENTRYPOINT ["vstone"]`.
  3. `dev` — runtime + Rust toolchain, `ruff`, `pytest`, `make`; agents run `make check` here.
- `requirements.lock` — every Python dependency pinned with hashes (`pip-compile --generate-hashes`).
- `rust-toolchain.toml` (pinned Rust version); `Cargo.lock` committed.
- `.dockerignore` — `target/`, `runs/`, `.venv/`, `.git/`.
- `docker-compose.yml` — services `vstone` (runtime) and `dev`; bind mounts `./work → /work`
  (configs, profiles, runs) and the repo → `/src` for `dev`.
- Makefile targets `docker-build` (arm64 and amd64 via `buildx`), `docker-check`, `docker-run CONFIG=…`.
- Run folders written as the host user's UID/GID (`--user $(id -u):$(id -g)`) so Finder and
  SolidWorks can open them without permission fixes.
- README: a "Run with Docker" section with the three make targets.

## Acceptance
- [ ] `make docker-build` builds `vstone:dev` and `vstone:latest` for `linux/arm64` on the MacBook
      (and `linux/amd64` via buildx; note each build time).
- [ ] `make docker-check` passes: Rust fmt/clippy/tests and Python ruff/pytest, all inside `dev`.
- [ ] `make docker-run CONFIG=configs/cylinder_18x60.toml` with `VSTONE_CORE_STUB=ring12` writes the
      full run folder into `./work/runs/` on the Mac, owned by the Mac user.
- [ ] Two clean-cache builds give the same `pip freeze` and the same `vstone-core --version`.
- [ ] Runtime image ≤ 1.5 GB (report the size).

## Out of scope
A long-running service or web API (comes with the UI, L3). GPU or Ollama inside Docker.

## Agent notes
