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

create-rust-optimizer-multi:
	docker buildx create --name rust-optimizer-multi

use-rust-optimizer-multi:
	$(MAKE) create-rust-optimizer-multi || true
	docker buildx use rust-optimizer-multi

build-rust-optimizer-amd64: use-rust-optimizer-multi
	docker buildx build --platform linux/amd64 -t $(DOCKER_NAME_RUST_OPTIMIZER):$(DOCKER_TAG) --target rust-optimizer --load .

build-rust-optimizer-arm64: use-rust-optimizer-multi
	docker buildx build --platform linux/arm64/v8 -t $(DOCKER_NAME_RUST_OPTIMIZER):$(DOCKER_TAG) --target rust-optimizer --load .

build-workspace-optimize-amd64: use-rust-optimizer-multi
	docker buildx build --platform linux/amd64 -t $(DOCKER_NAME_WORKSPACE_OPTIMIZER):$(DOCKER_TAG) --target workspace-optimizer --load .

build-workspace-optimize-amd64: use-rust-optimizer-multi
	docker buildx build --platform linux/arm64 -t $(DOCKER_NAME_WORKSPACE_OPTIMIZER):$(DOCKER_TAG) --target workspace-optimizer --load .

publish-rust-optimizer: use-rust-optimizer-multi
	docker buildx build --platform linux/amd64,linux/arm64/v8 -t $(DOCKER_NAME_RUST_OPTIMIZER):$(DOCKER_TAG) --target rust-optimizer --push .

publish-workspace-optimizer: use-rust-optimizer-multi
	docker buildx build --platform linux/amd64,linux/arm64/v8 -t $(DOCKER_NAME_WORKSPACE_OPTIMIZER):$(DOCKER_TAG) --target workspace-optimizer --push .

publish: publish-rust-optimizer publish-workspace-optimizer
