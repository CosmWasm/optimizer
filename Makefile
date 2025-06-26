# Docker names (DN) for the images
DN_OPTIMIZER := "cosmwasm/optimizer"
DN_RUST_OPTIMIZER := "cosmwasm/rust-optimizer"
DN_WORKSPACE_OPTIMIZER := "cosmwasm/workspace-optimizer"
DOCKER_TAG := 0.17.0

# Native arch
BUILDARCH := $(shell uname -m)

# Build the native CPU arch images
.PHONY: build
build: build-$(BUILDARCH)

.PHONY: build-x86_64
build-x86_64:
	docker buildx build --pull --platform linux/amd64 -t $(DN_OPTIMIZER):$(DOCKER_TAG) --target rust-optimizer --load .
	docker tag $(DN_OPTIMIZER):$(DOCKER_TAG) $(DN_RUST_OPTIMIZER):$(DOCKER_TAG)
	docker tag $(DN_OPTIMIZER):$(DOCKER_TAG) $(DN_WORKSPACE_OPTIMIZER):$(DOCKER_TAG)

.PHONY: build-arm64
build-arm64:
	docker buildx build --pull --platform linux/arm64/v8 -t $(DN_OPTIMIZER)-arm64:$(DOCKER_TAG) --target rust-optimizer --load .
	docker tag $(DN_OPTIMIZER)-arm64:$(DOCKER_TAG) $(DN_RUST_OPTIMIZER)-arm64:$(DOCKER_TAG)
	docker tag $(DN_OPTIMIZER)-arm64:$(DOCKER_TAG) $(DN_WORKSPACE_OPTIMIZER)-arm64:$(DOCKER_TAG)

.PHONY: publish-x86_64
publish-x86_64: build-x86_64
	docker push $(DN_OPTIMIZER):$(DOCKER_TAG)
	docker push $(DN_RUST_OPTIMIZER):$(DOCKER_TAG)
	docker push $(DN_WORKSPACE_OPTIMIZER):$(DOCKER_TAG)

.PHONY: publish-arm64
publish-arm64: build-arm64
	docker push $(DN_OPTIMIZER)-arm64:$(DOCKER_TAG)
	docker push $(DN_RUST_OPTIMIZER)-arm64:$(DOCKER_TAG)
	docker push $(DN_WORKSPACE_OPTIMIZER)-arm64:$(DOCKER_TAG)
