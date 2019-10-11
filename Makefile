.PHONY: build publish run debug

DOCKER_TAG := 1.38
CODE ?= "/path/to/contract"

build:
	docker build . -t confio/cosmwasm-opt:$(DOCKER_TAG)

publish:
	echo $(CODE)

# Usage: make run CODE=/path/to/contract
run:
	docker run --rm -v "$(CODE)":/code confio/cosmwasm-opt:$(DOCKER_TAG)

debug:
	docker run --rm -it confio/cosmwasm-opt:$(DOCKER_TAG) /bin/bash
