CARGO = cargo
RUNNER = sudo

RUST_LOG = RUST_LOG=info

RUN_ENV = $(RUST_LOG)

RUN_ARGS = -i enp1s0

DEBUG = target/debug/router
RELEASE = target/release/router

ROUTER_HOST = router
ROUTER_CONFIG = ./router-config.yaml

RUN_SCRIPT = ./router.sh

# build router
build: build-user build-bpf

build-user:
	$(CARGO) xtask build --release

build-bpf:
	$(CARGO) xtask build-ebpf --release


# copy and deploy the router
ship:
	scp $(RELEASE) $(ROUTER_HOST):~
	scp $(ROUTER_CONFIG) $(ROUTER_HOST):~
	scp $(RUN_SCRIPT) $(ROUTER_HOST):xdp_router
	ssh $(ROUTER_HOST) chmod u+x xdp_router
