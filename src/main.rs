mod ioctls;
mod nvlist;

use std::error::Error;
use std::ffi::CStr;
use std::fs::File;

fn ioc_pool_configs(z: &mut File) -> Result<nvlist::Header, Box<dyn Error>> {
    let mut zc: ioctls::Command = Default::default();
    let buf: [u8; 262144] = [0; 262144];
    zc.nvlist_dst = buf.as_ptr();
    zc.nvlist_dst_size = 262144;
    ioctls::ZFS_IOC_POOL_CONFIGS.ioctl(z, &mut zc).unwrap();
    let nvbuf = &buf[0..zc.nvlist_dst_size as usize];
    Ok(nvlist::unpack(nvbuf)?)
}

fn ioc_pool_stats(z: &mut File, pool: &CStr) -> Result<nvlist::Header, Box<dyn Error>> {
    let mut zc: ioctls::Command = Default::default();
    let name = pool.to_bytes_with_nul();
    zc.name[..name.len()].copy_from_slice(&name);
    let buf: [u8; 262144] = [0; 262144];
    zc.nvlist_dst = buf.as_ptr();
    zc.nvlist_dst_size = 262144;
    ioctls::ZFS_IOC_POOL_STATS.ioctl(z, &mut zc).unwrap();
    let nvbuf = &buf[0..zc.nvlist_dst_size as usize];
    Ok(nvlist::unpack(nvbuf)?)
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut z = File::open("/dev/zfs").unwrap_or_else(|_| std::process::exit(0));

    let configs = ioc_pool_configs(&mut z)?;

    for pool in configs.list.pairs.keys() {
        println!("pool: {:?}", pool);
        let stats = ioc_pool_stats(&mut z, pool)?;
        println!("{:#?}", stats);
    }

    Ok(())
}
