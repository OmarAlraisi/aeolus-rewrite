# aeolus

<h7 style="color: grey">Ancient Greek: Αἴολος, Aiolos - In Greek mythology, Aeolus was the ruler of the winds.</h7>

A layer 4 switching load balancer.

## Run

### Build eBPF app:

```BASH
cargo xtask build-ebpf
```

### Build user space app:

```BASH
cargo xtask run
```

*The default interface is `wlp1s0`. - Add use the interface flag for different interfaces. `-i <interface name>`*
*To view the logs you also need to add the `RUST_LOG` environment variable and set it to the required level.*

For example if you want to view logs at the `info` level and attach the eBPF application to interface eth1, run the following command:

```BASH
RUST_LOG=info cargo xtask run -- -i eth1
```
