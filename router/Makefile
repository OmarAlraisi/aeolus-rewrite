CARGO = cargo
RUNNER = sudo

RUST_LOG = info

DEBUG = target/debug/router
RELEAS = target/release/router

# build router
build: build-user build-bpf

build-user:
	$(CARGO) xtask build --release

build-bpf:
	$(CARGO) xtask build-ebpf --release


# copy and deploy the router
ship:
	scp $(RELEASE) $(ROUTER_HOST)
	ssh $(ROUTER_HOST) sudo -S ./router
