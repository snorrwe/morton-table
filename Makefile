build:
	cargo build

save-bench:
	cargo bench --bench quadtree_benches -- --save-baseline master

bench:
	cargo bench --bench quadtree_benches -- --baseline master

