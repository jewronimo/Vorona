.PHONY: check core-test py-test dev

PYTHON ?= python

check: core-test py-test

core-test:
	cd core && cargo fmt --check && cargo clippy -- -D warnings && cargo test

py-test:
	$(PYTHON) -m ruff check py tests/py
	$(PYTHON) -m pytest -q

dev:
	$(PYTHON) -m maturin develop --features python
