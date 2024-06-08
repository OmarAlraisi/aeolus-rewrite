#![no_std]
#![no_main]

use core::mem;

use aya_ebpf::{
    bindings::xdp_action,
    macros::{map, xdp},
    maps::{Array, HashMap},
    programs::XdpContext,
};
use network_types::eth::EthHdr;
// use aya_log_ebpf::info;

#[map]
static LISTENING_PORTS: HashMap<u16, u16> = HashMap::with_max_entries(512, 0);

#[map]
static SERVERS: Array<[u8; 6]> = Array::with_max_entries(170, 0);

#[map]
static SERVERS_COUNT: Array<u8> = Array::with_max_entries(1, 0);

#[map]
static CURRENT_COUNT: Array<u8> = Array::with_max_entries(1, 0);

#[xdp]
pub fn router(ctx: XdpContext) -> u32 {
    match try_router(ctx) {
        Ok(ret) => ret,
        Err(_) => xdp_action::XDP_ABORTED,
    }
}

#[inline(always)]
fn ptr_at<T>(ctx: XdpContext, offset: usize) -> Result<*const T, ()> {
    let start = ctx.data();
    let end = ctx.data_end();
    let size = mem::size_of::<T>();

    if start + offset + size > end {
        Err(())
    } else {
        Ok((start + offset) as *const T)
    }
}

#[inline(always)]
fn ptr_at_mut<T>(ctx: XdpContext, offset: usize) -> Result<*mut T, ()> {
    let ptr: *const T = ptr_at(ctx, offset)?;
    Ok(ptr as *mut T)
}

#[inline(always)]
fn get_dest_addr() -> [u8; 6] {
    if let Some(servers_count) = SERVERS_COUNT.get(0) {
        let servers_count = *servers_count as u8;
        if let Some(dest_idx) = CURRENT_COUNT.get_ptr_mut(0) {
            // let dst_addr =
            if let Some(dest_addr) = SERVERS.get(dest_idx as u32) {
                unsafe {
                    *dest_idx = (*dest_idx + 1) % servers_count as u8;
                }

                // unsafe { *eth_hdr }.dst_addr = *dst_addr;
                *dest_addr
            } else {
                [0u8; 6]
            }
        } else {
            [0; 6]
        }
    } else {
        [0; 6]
    }
}

fn try_router(ctx: XdpContext) -> Result<u32, ()> {
    // info!(&ctx, "received a packet");
    let eth_hdr: *mut EthHdr = ptr_at_mut(ctx, 0)?;
    let dest_addr = get_dest_addr();

    unsafe {
        (*eth_hdr).dst_addr = dest_addr;
    }

    Ok(xdp_action::XDP_TX)
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    unsafe { core::hint::unreachable_unchecked() }
}
