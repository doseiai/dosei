build:
	cargo build --release

dev:
	cargo run --bin doseid -- --config-path dev.doseid.toml

install:
	./install.sh

lint:
	cargo fmt
	cargo clippy --release --all-targets --all-features -- -D clippy::all

migrate:
	cd doseid && cargo sqlx migrate run

prepare:
	cd doseid && cargo sqlx prepare
