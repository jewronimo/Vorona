.PHONY: check core-test py-test dev dev-image dev-check dev-shell

check: core-test py-test

core-test:
	cd core && cargo fmt --check && cargo clippy -- -D warnings && cargo test

py-test:
	ruff check py && pytest -q

dev:
	maturin develop

DEV_RUN = docker run --rm --name vstone-dev-run --cpus=4 --memory=4g \
 -v "$(CURDIR)":/src \
 -v vstone-cargo-target:/src/core/target \
 -v vstone-cargo-registry:/usr/local/cargo/registry \
 vstone-dev:latest

dev-image:
	docker build -f Dockerfile.dev -t vstone-dev:latest .

dev-check:
	$(DEV_RUN) bash -c 'pip install --no-deps -e . && make check'

dev-shell:
	docker run --rm -it --name vstone-dev-shell --cpus=4 --memory=4g \
 -v "$(CURDIR)":/src -v vstone-cargo-target:/src/core/target \
 -v vstone-cargo-registry:/usr/local/cargo/registry vstone-dev:latest bash
