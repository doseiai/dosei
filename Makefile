default: build

build:
	cargo build --release

run-dev:
	docker compose -f docker-compose.yaml up

install-hobby:
	@/bin/bash -c './scripts/install_hobby.sh'

run-hobby:
	@/bin/bash -c './scripts/check_hobby_install.sh'
	docker compose -f docker-compose.hobby.yaml up
