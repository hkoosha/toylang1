.DEFAULT_GOAL = run

.PHONY: c
c:
	@sh -c 'for i in $$(seq 0 100); do echo; done'	

.PHONY: remake
remake:
	$(EDITOR) Makefile



.PHONY: build
build: c
	cargo build

.PHONY: run
run: c
	@cargo run

.PHONY: clippy
clippy: c
	cargo clippy --tests

.PHONY: format
format:
	cargo fmt

.PHONY: fmt
fmt: format

.PHONY: backtrace
backtrace: c
	RUST_BACKTRACE=1 cargo run 2>&1 | grep -v core:: | grep -v 'at /rustc'

.PHONY: test
test: c
	RUST_BACKTRACE=1 cargo test -- --show-output

.PHONY: verify
verify: c
	cargo test

.PHONY: clean
clean:
	cargo clean
