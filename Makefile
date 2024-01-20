build:
	cargo build --release

dev:
	cargo run --bin doseid

install:
	./install.sh

lint:
	cargo fmt
	cargo clippy --release --all-targets --all-features -- -D clippy::all

prepare:
	cd doseid && cargo sqlx prepare
