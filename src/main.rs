mod ioctls;
mod nvlist;

use std::error::Error;
use std::ffi::{CStr,CString};
use std::fs::File;

fn zc_new() -> (ioctls::Command, Box<[u8; 262144]>) {
    let mut zc: ioctls::Command = Default::default();
    let buf: Box<[u8; 262144]> = Box::new([0; 262144]);
    zc.nvlist_dst = buf.as_ptr();
    zc.nvlist_dst_size = 262144;
    (zc, buf)
}

fn ioc_pool_configs(z: &mut File) -> Result<nvlist::Header, Box<dyn Error>> {
    let (mut zc, mut buf) = zc_new();
    ioctls::ZFS_IOC_POOL_CONFIGS.ioctl(z, &mut zc).unwrap();
    let nvbuf = &buf[0..zc.nvlist_dst_size as usize];
    Ok(nvlist::unpack(nvbuf)?)
}

fn ioc_pool_stats(z: &mut File, pool: &CStr) -> Result<nvlist::Header, Box<dyn Error>> {
    let (mut zc, mut buf) = zc_new();
    let name = pool.to_bytes_with_nul();
    zc.name[..name.len()].copy_from_slice(&name);
    ioctls::ZFS_IOC_POOL_STATS.ioctl(z, &mut zc);
    let nvbuf = &buf[0..zc.nvlist_dst_size as usize];
    Ok(nvlist::unpack(nvbuf)?)
}

fn ioc_objset_stats(z: &mut File, objset: &CStr) -> Result<nvlist::Header, Box<dyn Error>> {
    let (mut zc, mut buf) = zc_new();
    let name = objset.to_bytes_with_nul();
    zc.name[..name.len()].copy_from_slice(&name);
    ioctls::ZFS_IOC_OBJSET_STATS.ioctl(z, &mut zc)?;
    let nvbuf = &buf[0..zc.nvlist_dst_size as usize];
    Ok(nvlist::unpack(nvbuf)?)
}

fn ioc_dataset_list_next(z: &mut File, dataset: &CStr, cookie: u64) -> Result<(CString, nvlist::Header, u64), Box<dyn Error>> {
    let (mut zc, mut buf) = zc_new();
    let name = dataset.to_bytes_with_nul();
    zc.name[..name.len()].copy_from_slice(&name);
    zc.cookie = cookie;
    ioctls::ZFS_IOC_DATASET_LIST_NEXT.ioctl(z, &mut zc)?;
    let nvbuf = &buf[0..zc.nvlist_dst_size as usize];
    Ok((CStr::from_bytes_until_nul(&zc.name)?.into(), nvlist::unpack(nvbuf)?, zc.cookie))
}

fn print_dataset(dataset: &CStr, stats: nvlist::Header) -> Result<(), Box<dyn Error>> {
    if let nvlist::Data::List(ref l) = stats.list.pairs[&CString::new("used")?] {
        if let nvlist::Data::UInt64(ref u) = l.pairs[&CString::new("value")?] {
            println!("{:?} {:?}", dataset, u);
        }
    }
    Ok(())
}

fn iter_dataset(z: &mut File, dataset: &CStr) -> Result<(), Box<dyn Error>> {
    let mut cookie = 0;
    while let Ok((name, stats, next_cookie)) = ioc_dataset_list_next(z, &dataset, cookie) {
        print_dataset(&name, stats)?;
        iter_dataset(z, &name)?;
        cookie = next_cookie;
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut z = File::open("/dev/zfs").unwrap_or_else(|_| std::process::exit(0));

    let configs = ioc_pool_configs(&mut z)?;

    for pool in configs.list.pairs.keys() {
        let stats = ioc_objset_stats(&mut z, pool)?;
        print_dataset(pool, stats)?;
        iter_dataset(&mut z, pool);
    }

    Ok(())
}
