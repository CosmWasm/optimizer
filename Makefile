.PHONY: build-rust-optimizer build-workspace-optimizer build create-rust-optimizer-builder use-rust-optimizer-builder publish-rust-optimizer publish-workspace-optimizer publish

DOCKER_NAME_RUST_OPTIMIZER := "cosmwasm/rust-optimizer"
DOCKER_NAME_WORKSPACE_OPTIMIZER := "cosmwasm/workspace-optimizer"
DOCKER_TAG := 0.12.2

# Build images locally for the host CPU architecture

build-rust-optimizer:
	docker build -t $(DOCKER_NAME_RUST_OPTIMIZER):$(DOCKER_TAG) --target rust-optimizer .

build-workspace-optimizer:
	docker build -t $(DOCKER_NAME_WORKSPACE_OPTIMIZER):$(DOCKER_TAG) --target workspace-optimizer .

build: build-rust-optimizer build-workspace-optimizer

# Build multi-CPU architecture images and publish them. rust alpine images support the linux/amd64 and linux/arm64/v8 architectures.

create-rust-optimizer-builder:
	docker buildx create --name rust-optimizer-builder

# create-rust-optimizer-builder must be run before running use-rust-optimizer-builder for the first time
use-rust-optimizer-builder:
	docker buildx use rust-optimizer-builder

publish-rust-optimizer: use-rust-optimizer-builder
	docker buildx build --platform linux/amd64,linux/arm64/v8 -t $(DOCKER_NAME_RUST_OPTIMIZER):$(DOCKER_TAG) --target rust-optimizer --push .

publish-workspace-optimizer: use-rust-optimizer-builder
	docker buildx build --platform linux/amd64,linux/arm64/v8 -t $(DOCKER_NAME_WORKSPACE_OPTIMIZER):$(DOCKER_TAG) --target workspace-optimizer --push .

publish: publish-rust-optimizer publish-workspace-optimizer
