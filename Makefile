.PHONY: check core-test py-test dev dev-image dev-up dev-down dev-check dev-shell acceptance

check: core-test py-test

core-test:
	cd core && cargo fmt --check && cargo clippy -- -D warnings && cargo test

py-test:
	ruff check py && pytest -q

dev:
	maturin develop

DEV_CONTAINER = Vorona-dev
# Resolve the actual bind mount, including nested git worktrees.
DEV_WORKDIR = $(shell python3 scripts/container_workdir.py "$(DEV_CONTAINER)" "$(CURDIR)")

dev-image:
	docker build -f Dockerfile.dev -t vstone-dev:latest .

# Persistent container: survives between `make` invocations, so dev-check
# doesn't pay container-creation + reinstall cost every run, and you can
# `make dev-shell` into the exact same container a failing dev-check just used.
dev-up:
	@docker ps -q -f name=^/$(DEV_CONTAINER)$$ | grep -q . || \
	docker run -d --name $(DEV_CONTAINER) --cpus=4 --memory=4g \
	 -v "$(CURDIR)":/src \
	 -v vstone-cargo-target:/src/core/target \
	 -v vstone-cargo-registry:/usr/local/cargo/registry \
	 vstone-dev:latest tail -f /dev/null

dev-down:
	docker rm -f $(DEV_CONTAINER) >/dev/null 2>&1 || true

dev-check: dev-up
	@test -n "$(DEV_WORKDIR)"
	docker exec -w "$(DEV_WORKDIR)" $(DEV_CONTAINER) bash -c 'pip install --no-deps -e . && make check'

dev-shell: dev-up
	docker exec -it -w "$(DEV_WORKDIR)" $(DEV_CONTAINER) bash

# Host entrypoint; all builds, tests and measurements execute in Docker.
acceptance: dev-up
	@test -n "$(DEV_WORKDIR)"
	bash scripts/acceptance_host.sh "$(DEV_CONTAINER)" "$(DEV_WORKDIR)"
