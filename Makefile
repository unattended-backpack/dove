# Configuration is loaded from `.env.maintainer` and can be overridden by
# environment variables.
#
# Usage:
#   make build                    # Build using `.env.maintainer`.
#   BUILD_IMAGE=... make build    # Override specific variables.

# Load configuration from `.env.maintainer` if it exists.
-include .env.maintainer

# Allow environment variable overrides with defaults.
BUILD_IMAGE ?= unattended/petros:latest
RUNTIME_IMAGE ?= debian:trixie-slim
DOCKER_BUILD_ARGS ?=
DOCKER_RUN_ARGS ?=
DOVE_NAME ?= dove
IMAGE_TAG ?= latest
ACT_PULL ?= false

.PHONY: clean
clean:
	@bash -c 'echo -e "\033[33mWARNING: This will delete build artifacts.\033[0m"; \
	read -p "Are you sure you want to continue? [y/N]: " confirm; \
	if [[ "$$confirm" != "y" && "$$confirm" != "Y" ]]; then \
		echo "Operation cancelled."; \
		exit 1; \
	fi'
	rm -rf out/
	rm -rf target/
	rm -f result result-*

.PHONY: build
build:
	@echo "Building native artifacts ..."
	mkdir -p out
	cargo build --release --bin dove
	cp ./target/release/dove ./out/dove
	@echo "Build complete."

.PHONY: preen-test
preen-test:
	cargo build && cd preen-tests && bash test.sh && cd ../

.PHONY: peck-test
peck-test:
	cargo build && cd peck-tests && bash test.sh && cd ../

.PHONY: test
test:
	@echo "Running tests ..."
	cargo build && cargo test
	$(MAKE) preen-test
	$(MAKE) peck-test
	@echo "... tests completed."

.PHONY: docker-d
docker-d:
	@echo "Building Dove image ..."
	@echo "  Build image:   $(BUILD_IMAGE)"
	@echo "  Runtime image: $(RUNTIME_IMAGE)"
	@echo "  Output tag:    $(DOVE_NAME):$(IMAGE_TAG)"
	@mkdir -p out
	docker build \
		$(DOCKER_BUILD_ARGS) \
		--build-arg BUILD_IMAGE=$(BUILD_IMAGE) \
		--build-arg RUNTIME_IMAGE=$(RUNTIME_IMAGE) \
		-f Dockerfile.dove \
		-t $(DOVE_NAME):$(IMAGE_TAG) \
		.
	@echo "Build complete: $(DOVE_NAME):$(IMAGE_TAG)"

.PHONY: docker
docker:
	$(MAKE) docker-d

.PHONY: ci
ci:
	@echo "Building images from pre-built binaries (CI mode) ..."
	@if [ ! -f out/dove ]; then \
		echo "ERROR: Pre-built binaries not found in ./out/" >&2; \
		echo "Run 'make build' first to create the binaries." >&2; \
		exit 1; \
	fi
	@echo "  Build image:   $(BUILD_IMAGE)"
	@echo "  Runtime image: $(RUNTIME_IMAGE)"
	@echo "  Output tag:    $(DOVE_NAME):$(IMAGE_TAG)"
	docker build \
		$(DOCKER_BUILD_ARGS) \
		--build-arg BUILD_TYPE=prebuilt \
		--build-arg BUILD_IMAGE=$(BUILD_IMAGE) \
		--build-arg RUNTIME_IMAGE=$(RUNTIME_IMAGE) \
		-f Dockerfile.dove \
		-t $(DOVE_NAME):$(IMAGE_TAG) \
		.
	@echo "Build complete: $(DOVE_NAME):$(IMAGE_TAG)"

.PHONY: run-d
run-d:
	@echo "Starting Dove container ..."
	docker run --rm -it \
		--name $(DOVE_NAME) \
		$(DOCKER_RUN_ARGS) \
		$(DOVE_NAME):$(IMAGE_TAG)

.PHONY: stop-d
stop-d:
	@echo "Stopping Dove container..."
	docker stop $(DOVE_NAME)
	docker rm $(DOVE_NAME) || true

.PHONY: shell-d
shell-d:
	@echo "Opening shell in Dove ..."
	docker run --rm -it \
		--entrypoint /bin/bash \
		$(DOVE_NAME):$(IMAGE_TAG)

.PHONY: run
run:
	$(MAKE) build
	$(MAKE) ci
	$(MAKE) run-d

.PHONY: act
act:
	@echo "Running GitHub Actions workflow locally with act ..."
	@if [ ! -d ".act-secrets" ]; then \
		echo "WARNING: .act-secrets/ directory not found" >&2; \
		echo "See docs/WORKFLOW_TESTING.md for setup instructions" >&2; \
	fi
	@echo "Cleaning previous act artifacts to prevent cross-repo contamination ..."
	@rm -rf /tmp/act-artifacts/*
	@echo "Setting up temporary secrets mount ..."
	@sudo mkdir -p /opt/github-runner
	@sudo rm -rf /opt/github-runner/secrets
	@sudo ln -s $(CURDIR)/.act-secrets /opt/github-runner/secrets
	@trap "sudo rm -f /opt/github-runner/secrets" EXIT; \
	DOCKER_HOST="" act push -W .github/workflows/release.yml \
		--container-options "-v /opt/github-runner/secrets:/opt/github-runner/secrets:ro" \
		--artifact-server-path=/tmp/act-artifacts \
		--pull=$(ACT_PULL) \
		$(if $(DOCKER_BUILD_ARGS),--env DOCKER_BUILD_ARGS="$(DOCKER_BUILD_ARGS)")

.PHONY: help
help:
	@echo "Build System"
	@echo ""
	@echo "Targets:"
	@echo "  clean           Clean output directories."
	@echo "  build           Build native binaries."
	@echo "  preen-test      Test the preen utility."
	@echo "  peck-test       Test the peck utility."
	@echo "  test            Run all tests for the build."
	@echo "  docker-c        Build just the Dove image."
	@echo "  docker          Build Docker images (compiles inside container)."
	@echo "  ci              Build Docker images from pre-built binaries."
	@echo "  run-c           Run the built Dove image locally."
	@echo "  run             Run the built Docker images locally."
	@echo "  stop-c          Stop the running Dove container."
	@echo "  shell-c         Open a shell in the Dove image."
	@echo "  act             Test GitHub Actions release workflow locally."
	@echo "  help            Show this help message."
	@echo ""
	@echo "Configuration:"
	@echo "  Variables are loaded from .env.maintainer."
	@echo "  Override with environment variables:"
	@echo "    BUILD_IMAGE        - Builder image."
	@echo "    RUNTIME_IMAGE      - Runtime base image."
	@echo "    DOVE_NAME          - Dove image name."
	@echo "    IMAGE_TAG          - Docker image tag."
	@echo "    DOCKER_BUILD_ARGS  - Additional Docker build flags."
	@echo "    DOCKER_RUN_ARGS    - Additional Docker run flags."
	@echo ""
	@echo "Examples:"
	@echo "  make build"
	@echo "  BUILD_IMAGE=unattended/petros:latest make build"
	@echo "  IMAGE_TAG=v1.0.0 make build"
	@echo "  DOCKER_BUILD_ARGS='--network host' make build"
	@echo "  DOCKER_RUN_ARGS='--network host' make run-d"

.DEFAULT_GOAL := build
