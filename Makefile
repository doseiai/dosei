build:
	cargo build --release

dev:
	cargo run --bin doseid -- --config-path dev.doseid.toml

dev-compose:
	docker compose -f docker-compose.base.yml up --build

test-compose:
	docker compose -f docker-compose.test.yml up --build

configure-compose:
	./script/configure-compose.sh

lint:
	cargo fmt
	cargo clippy --release --all-targets --all-features -- -D clippy::all

migrate:
	cd doseid && cargo sqlx migrate run

prepare:
	cd doseid && cargo sqlx prepare
