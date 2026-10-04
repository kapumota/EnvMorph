.PHONY: build test integration workload shell-syntax check clean

ENVMORPH_BIN ?= $(abspath target/release/envmorph)

build:
	cargo build --release

test:
	cargo test

integration: build
	ENVMORPH_BIN=$(ENVMORPH_BIN) bash tests/integration.sh

workload:
	cd workloads/literature-pipeline/source && bash tests/test_pipeline.sh

shell-syntax:
	find tests workloads -type f -name '*.sh' -print0 | xargs -0 -r -n1 bash -n

check: test integration workload shell-syntax

clean:
	cargo clean
	rm -rf .envmorph .demo results
