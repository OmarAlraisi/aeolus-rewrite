CARGO = cargo
RUNNER = sudo

RUST_LOG = RUST_LOG=info

RUN_ENV = $(RUST_LOG)

DEBUG = target/debug/aeolus
RELEASE = target/release/aeolus

NODE_1_HOST = node1
NODE_2_HOST = node2


all: build
	$(RUNNER) $(RUST_LOG) $(RELEASE)

# Realese build
build: build-user build-ebpf

build-user: aeolus/src/main.rs
	$(CARGO) xtask build --release

build-ebpf: aeolus-ebpf/src/main.rs
	$(CARGO) xtask build-ebpf --release



debug: build-debug
	$(RUNNER) $(RUST_LOG) $(DEBUG)

# Debug build
build-debug: build-user-debug build-ebpf-debug

build-user-debug: aeolus/src/main.rs
	$(CARGO) xtask build

build-ebpf-debug: aeolus-ebpf/src/main.rs
	$(CARGO) xtask build-ebpf


ship:
	scp $(RELEASE) $(NODE_1_HOST):~
	scp $(RELEASE) $(NODE_2_HOST):~
	ssh $(RUN_ENV) $(NODE_1_HOS) sudo -S ./aeolus
	ssh $(RUN_ENV) $(NODE_2_HOS) sudo -S ./aeolus

ship-debug:
	scp $(DEBUG) $(NODE_1_HOST):~
	scp $(DEBUG) $(NODE_2_HOST):~
	ssh $(RUN_ENV) $(NODE_1_HOS) sudo -S ./aeolus
	ssh $(RUN_ENV) $(NODE_2_HOS) sudo -S ./aeolus
