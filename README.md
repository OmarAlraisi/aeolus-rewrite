# <span style="text-transform: uppercase; font-size: 28px"><span style="font-size: 36px">a</span>eolus</span>

<p style="color: grey; font-size: 16px; margin-top: -10px;">Ancient Greek: Αἴολος, Aiolos - In Greek mythology, Aeolus was the ruler of the winds.</p>

Aeolus is an **[`eBPF`](https://ebpf.io/what-is-ebpf/)** layer 4 switching load balancer built using Rust and **[`Aya-rs`](https://github.com/aya-rs/aya)**. It utilizes the **[`eXpress-Data-Path`](https://en.wikipedia.org/wiki/Express_Data_Path)** (XDP) for high-performance packet inspection and processing.

## Key features:

1. Fast packet processing.
2. Scales well for high-traffic applications.
3. Performs health checks.
4. Allows for connection draining.

## Usage:

Check out the **[`setup`](./docs/setup.md)** documentation before proceeding.

1. Release mode:

Run `make build` to build Aeolus and `make ship` to copy it to the target servers.

2. Debug mode:

Run `make build-debug` to build Aeolus and `make ship-debug` to copy it to the target servers.
