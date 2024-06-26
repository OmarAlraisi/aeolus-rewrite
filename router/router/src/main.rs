mod config;

use anyhow::Context;
use aya::programs::{Xdp, XdpFlags};
use aya::{
    include_bytes_aligned,
    maps::{Array, HashMap},
    Bpf,
};
use aya_log::BpfLogger;
use clap::Parser;
use config::Config;
use log::{debug, info, warn};
use tokio::signal;

#[derive(Debug, Parser)]
struct Opt {
    #[clap(short, long, default_value = "eth0")]
    iface: String,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let opt = Config::parse()?;

    env_logger::init();

    // Bump the memlock rlimit. This is needed for older kernels that don't use the
    // new memcg based accounting, see https://lwn.net/Articles/837122/
    let rlim = libc::rlimit {
        rlim_cur: libc::RLIM_INFINITY,
        rlim_max: libc::RLIM_INFINITY,
    };
    let ret = unsafe { libc::setrlimit(libc::RLIMIT_MEMLOCK, &rlim) };
    if ret != 0 {
        debug!("remove limit on locked memory failed, ret is: {}", ret);
    }

    // This will include your eBPF object file as raw bytes at compile-time and load it at
    // runtime. This approach is recommended for most serde = "1.0.203"real-world use cases. If you would
    // like to specify the eBPF program at runtime rather than at compile-time, you can
    // reach for `Bpf::load_file` instead.
    #[cfg(debug_assertions)]
    let mut bpf = Bpf::load(include_bytes_aligned!(
        "../../target/bpfel-unknown-none/debug/router"
    ))?;
    #[cfg(not(debug_assertions))]
    let mut bpf = Bpf::load(include_bytes_aligned!(
        "../../target/bpfel-unknown-none/release/router"
    ))?;
    if let Err(e) = BpfLogger::init(&mut bpf) {
        // This can happen if you remove all log statements from your eBPF program.
        warn!("failed to initialize eBPF logger: {}", e);
    }
    let program: &mut Xdp = bpf.program_mut("router").unwrap().try_into()?;
    program.load()?;
    program.attach(&opt.iface, XdpFlags::default())
        .context("failed to attach the XDP program with default flags - try changing XdpFlags::default() to XdpFlags::SKB_MODE")?;

    // setup maps
    let mut listeninig_ports: HashMap<_, u16, u16> =
        HashMap::try_from(bpf.map_mut("LISTENING_PORTS").unwrap())?;
    for port in opt.ports.iter() {
        listeninig_ports.insert(port, port, 0)?;
    }

    let mut servers: Array<_, [u8; 6]> = Array::try_from(bpf.take_map("SERVERS").unwrap())?;
    for (idx, server) in opt.servers.iter().enumerate() {
        servers.set(idx as u32, server, 0)?;
    }

    let mut servers_count: Array<_, u8> = Array::try_from(bpf.take_map("SERVERS_COUNT").unwrap())?;
    servers_count.set(0, opt.servers.len() as u8, 0)?;

    let mut current_count: Array<_, u8> = Array::try_from(bpf.take_map("CURRENT_COUNT").unwrap())?;
    current_count.set(0, 0, 0)?;


    info!("Waiting for Ctrl-C...");
    signal::ctrl_c().await?;
    info!("Exiting...");

    Ok(())
}
