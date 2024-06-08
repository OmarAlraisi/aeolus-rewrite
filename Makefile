CARGO = cargo
RUNNER = sudo

RUST_LOG = RUST_LOG=info

DEBUG = target/debug/aeolus
RELEASE = target/release/aeolus


all: build
	$(RUNNER) $(RUST_LOG) $(RELEASE)

# Realese build
build: build-user build-bpf

build-user: aeolus/src/main.rs
	$(CARGO) xtask build --release

build-bpf: aeolus-ebpf/src/main.rs
	$(CARGO) xtask build-ebpf --release



debug: build-debug
	$(RUNNER) $(RUST_LOG) $(DEBUG)

# Debug build
build-debug: build-user-debug build-bpf-debug

build-user-debug: aeolus/src/main.rs
	$(CARGO) xtask build

build-bpf-debug: aeolus-ebpf/src/main.rs
	$(CARGO) xtask build-ebpf
