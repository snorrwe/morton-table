.PHONY: build

build:
	cargo build

save-bench:
	cargo bench --bench quadtree_benches -- --save-baseline naive

bench:
	cargo bench --bench quadtree_benches -- --baseline naive

test:
	cargo test
	cargo test --benches

run_full:
	git checkout -d origin/naive
	cargo bench --bench quadtree_benches -- --save-baseline naive
	git checkout master
	cargo bench --bench quadtree_benches -- --baseline naive
