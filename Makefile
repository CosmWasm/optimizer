.PHONY: build publish run debug

DOCKER_TAG := 0.6.0
CODE ?= "/path/to/contract"
USER_ID := $(shell id -u)
USER_GROUP = $(shell id -g)

build:
	docker build . -t confio/cosmwasm-opt:$(DOCKER_TAG)

publish: build
	docker push confio/cosmwasm-opt:$(DOCKER_TAG)

# Usage: make run CODE=/path/to/contract
run:
	docker run --rm -u $(USER_ID):$(USER_GROUP) -v "$(CODE)":/code confio/cosmwasm-opt:$(DOCKER_TAG)

debug:
	docker run --rm -it -u $(USER_ID):$(USER_GROUP) -v "$(CODE)":/code confio/cosmwasm-opt:$(DOCKER_TAG) /bin/bash
