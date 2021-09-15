.PHONY: build-rust-optimizer build-workspace-optimizer build create-rust-optimizer-multi use-rust-optimizer-multi publish-rust-optimizer publish-workspace-optimizer publish

DOCKER_NAME_RUST_OPTIMIZER := "cosmwasm/rust-optimizer"
DOCKER_NAME_WORKSPACE_OPTIMIZER := "cosmwasm/workspace-optimizer"
DOCKER_TAG := 0.12.2_rust_port

# Native arch
BUILDARCH := $(shell uname -m)

# Build multi-CPU architecture images and publish them. rust alpine images support the linux/amd64 and linux/arm64/v8 architectures.
build: build-rust-optimizer build-workspace-optimizer

create-rust-optimizer-multi:
	docker context create rust-optimizer-multi
	docker buildx create --name rust-optimizer-multi rust-optimizer-multi

use-rust-optimizer-multi:
	$(MAKE) create-rust-optimizer-multi || true
	docker buildx use rust-optimizer-multi

build-rust-optimizer-x86_64: use-rust-optimizer-multi
	docker buildx build --platform linux/amd64 -t $(DOCKER_NAME_RUST_OPTIMIZER):$(DOCKER_TAG) --target rust-optimizer --load .

build-rust-optimizer-arm64: use-rust-optimizer-multi
	docker buildx build --platform linux/arm64/v8 -t $(DOCKER_NAME_RUST_OPTIMIZER):$(DOCKER_TAG) --target rust-optimizer --load .

build-workspace-optimizer-x86_64: use-rust-optimizer-multi
	docker buildx build --platform linux/amd64 -t $(DOCKER_NAME_WORKSPACE_OPTIMIZER):$(DOCKER_TAG) --target workspace-optimizer --load .

build-workspace-optimizer-arm64: use-rust-optimizer-multi
	docker buildx build --platform linux/arm64/v8 -t $(DOCKER_NAME_WORKSPACE_OPTIMIZER):$(DOCKER_TAG) --target workspace-optimizer --load .

# Build only the native version by default
build-rust-optimizer: build-rust-optimizer-$(BUILDARCH)

# Build only the native version by default
build-workspace-optimizer: build-workspace-optimizer-$(BUILDARCH)

publish-rust-optimizer: use-rust-optimizer-multi
	docker buildx build --platform linux/amd64,linux/arm64/v8 -t $(DOCKER_NAME_RUST_OPTIMIZER):$(DOCKER_TAG) --target rust-optimizer --push .

publish-workspace-optimizer: use-rust-optimizer-multi
	docker buildx build --platform linux/amd64,linux/arm64/v8 -t $(DOCKER_NAME_WORKSPACE_OPTIMIZER):$(DOCKER_TAG) --target workspace-optimizer --push .

publish: publish-rust-optimizer publish-workspace-optimizer
