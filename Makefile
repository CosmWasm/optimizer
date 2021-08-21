.PHONY: build-rust-optimizer build-workspace-optimizer build publish-rust-optimizer publish-workspace-optimizer publish

DOCKER_NAME_BASE_OPTIMIZER := "base-optimizer"
DOCKER_NAME_RUST_OPTIMIZER := "cosmwasm/rust-optimizer"
DOCKER_NAME_WORKSPACE_OPTIMIZER := "cosmwasm/workspace-optimizer"
DOCKER_TAG := 0.11.5

build-base-optimizer:
	docker build -t $(DOCKER_NAME_BASE_OPTIMIZER):$(DOCKER_TAG) --file base-optimizer.Dockerfile .
	docker tag $(DOCKER_NAME_BASE_OPTIMIZER):$(DOCKER_TAG) $(DOCKER_NAME_BASE_OPTIMIZER):latest

build-rust-optimizer: build-base-optimizer
	docker build -t $(DOCKER_NAME_RUST_OPTIMIZER):$(DOCKER_TAG) --pull=false --file rust-optimizer.Dockerfile .

build-workspace-optimizer: build-base-optimizer
	docker build -t $(DOCKER_NAME_WORKSPACE_OPTIMIZER):$(DOCKER_TAG) --pull=false --file workspace-optimizer.Dockerfile .

publish-rust-optimizer: build-rust-optimizer
	docker push $(DOCKER_NAME_RUST_OPTIMIZER):$(DOCKER_TAG)

publish-workspace-optimizer: build-workspace-optimizer
	docker push $(DOCKER_NAME_WORKSPACE_OPTIMIZER):$(DOCKER_TAG)

build: build-rust-optimizer build-workspace-optimizer

publish: publish-rust-optimizer publish-workspace-optimizer
