.DEFAULT_GOAL = run

.PHONY: c
c:
	@sh -c 'for i in $$(seq 0 100); do echo; done'	

.PHONY: build
build: c
	cargo build

.PHONY: run
run: c
	@cargo run

.PHONY: clippy
clippy: c
	cargo clippy

.PHONY: format
format:
	cargo fmt

.PHONY: fmt
fmt: format

.PHONY: backtrace
backtrace:
	RUST_BACKTRACE=1 cargo run 2>&1 | grep -v core:: | grep -v 'at /rustc'

