.PHONY: build-rust-optimizer build-workspace-optimizer build create-rust-optimizer-multi use-rust-optimizer-multi publish-rust-optimizer-multi publish-workspace-optimizer-multi publish-multi

DOCKER_NAME_RUST_OPTIMIZER := "cosmwasm/rust-optimizer"
DOCKER_NAME_WORKSPACE_OPTIMIZER := "cosmwasm/workspace-optimizer"
DOCKER_TAG := 0.12.11

# Native arch
BUILDARCH := $(shell uname -m)

# Build multi-CPU architecture images and publish them. rust alpine images support the linux/amd64 and linux/arm64/v8 architectures.
build: build-rust-optimizer build-workspace-optimizer

build-rust-optimizer-x86_64:
	docker buildx build --pull --platform linux/amd64 -t $(DOCKER_NAME_RUST_OPTIMIZER):$(DOCKER_TAG) --target rust-optimizer --load .

build-rust-optimizer-arm64:
	docker buildx build --pull --platform linux/arm64/v8 -t $(DOCKER_NAME_RUST_OPTIMIZER)-arm64:$(DOCKER_TAG) --target rust-optimizer --load .

build-workspace-optimizer-x86_64:
	docker buildx build --pull --platform linux/amd64 -t $(DOCKER_NAME_WORKSPACE_OPTIMIZER):$(DOCKER_TAG) --target workspace-optimizer --load .

build-workspace-optimizer-arm64:
	docker buildx build --pull --platform linux/arm64/v8 -t $(DOCKER_NAME_WORKSPACE_OPTIMIZER)-arm64:$(DOCKER_TAG) --target workspace-optimizer --load .

# Build only the native version by default
build-rust-optimizer: build-rust-optimizer-$(BUILDARCH)

# Build only the native version by default
build-workspace-optimizer: build-workspace-optimizer-$(BUILDARCH)

build-x86_64: build-rust-optimizer-x86_64 build-workspace-optimizer-x86_64
build-arm64: build-rust-optimizer-arm64 build-workspace-optimizer-arm64

publish-x86_64: build-x86_64
	docker push $(DOCKER_NAME_RUST_OPTIMIZER):$(DOCKER_TAG)
	docker push $(DOCKER_NAME_WORKSPACE_OPTIMIZER):$(DOCKER_TAG)

publish-arm64: build-arm64
	docker push $(DOCKER_NAME_RUST_OPTIMIZER)-arm64:$(DOCKER_TAG)
	docker push $(DOCKER_NAME_WORKSPACE_OPTIMIZER)-arm64:$(DOCKER_TAG)
