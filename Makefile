.PHONY: build publish run debug

DOCKER_NAME := "cosmwasm/rust-optimizer"
DOCKER_TAG := 0.9.1
CODE ?= "/path/to/contract"
USER_ID := $(shell id -u)
USER_GROUP = $(shell id -g)

build:
	docker build . -t $(DOCKER_NAME):$(DOCKER_TAG)

publish: build
	docker push $(DOCKER_NAME):$(DOCKER_TAG)

# Usage: make run CODE=/path/to/contract
run:
	docker run --rm -u $(USER_ID):$(USER_GROUP) -v "$(CODE)":/code $(DOCKER_NAME):$(DOCKER_TAG)

debug:
	docker run --rm -it -u $(USER_ID):$(USER_GROUP) -v "$(CODE)":/code $(DOCKER_NAME):$(DOCKER_TAG) /bin/bash
