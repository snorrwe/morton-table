.PHONY: build

build:
	cargo build

save-bench:
	cargo bench --bench quadtree_benches -- --save-baseline master

bench:
	cargo bench --bench quadtree_benches -- --baseline master

test:
	cargo check --benches
	cargo test
	cargo test --benches
