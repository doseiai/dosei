lint:
	cargo fmt
	cargo clippy --release --all-targets --all-features -- -D clippy::all

prepare:
	cd doseid && cargo sqlx prepare
