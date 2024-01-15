build:
	cargo build --release

install:
	./install.sh

lint:
	cargo fmt
	cargo clippy --release --all-targets --all-features -- -D clippy::all

prepare:
	cd doseid && cargo sqlx prepare
