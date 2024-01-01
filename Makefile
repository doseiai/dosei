install:
	git clone git@github.com:doseiai/engine.git && cd engine && ./install.sh

run:
	docker compose -f engine/docker-compose.hobby.x86.yaml up

run-arm:
	docker compose -f engine/docker-compose.hobby.arm.yaml up