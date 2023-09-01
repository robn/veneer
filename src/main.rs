mod sys;
mod nvpair;

use std::error::Error;
use std::ffi::{CStr,CString};
use std::fs::File;
use nvpair::{Pair,List};
use hexdump::hexdump;

#[macro_use]
extern crate num_derive;

fn zc_new() -> (sys::ZFSCommand, Box<[u8; 262144]>) {
    let mut zc: sys::ZFSCommand = Default::default();
    let buf: Box<[u8; 262144]> = Box::new([0; 262144]);
    zc.nvlist_dst = buf.as_ptr();
    zc.nvlist_dst_size = 262144;
    (zc, buf)
}

fn ioc_pool_configs(z: &mut File) -> Result<List, Box<dyn Error>> {
    let (mut zc, mut buf) = zc_new();
    sys::ZFS_IOC_POOL_CONFIGS.ioctl(z, &mut zc).unwrap();
    let nvbuf = &buf[0..zc.nvlist_dst_size as usize];
	hexdump(&nvbuf);
    Ok(nvpair::parse(nvbuf)?)
}

fn ioc_pool_stats(z: &mut File, pool: &CStr) -> Result<List, Box<dyn Error>> {
    let (mut zc, mut buf) = zc_new();
    let name = pool.to_bytes_with_nul();
    zc.name[..name.len()].copy_from_slice(&name);
    sys::ZFS_IOC_POOL_STATS.ioctl(z, &mut zc);
    let nvbuf = &buf[0..zc.nvlist_dst_size as usize];
    Ok(nvpair::parse(nvbuf)?)
}

fn ioc_pool_get_props(z: &mut File, pool: &CStr) -> Result<List, Box<dyn Error>> {
    let (mut zc, mut buf) = zc_new();
    let name = pool.to_bytes_with_nul();
    zc.name[..name.len()].copy_from_slice(&name);
    sys::ZFS_IOC_POOL_GET_PROPS.ioctl(z, &mut zc);
    let nvbuf = &buf[0..zc.nvlist_dst_size as usize];
    Ok(nvpair::parse(nvbuf)?)
}

fn ioc_objset_stats(z: &mut File, objset: &CStr) -> Result<List, Box<dyn Error>> {
    let (mut zc, mut buf) = zc_new();
    let name = objset.to_bytes_with_nul();
    zc.name[..name.len()].copy_from_slice(&name);
    sys::ZFS_IOC_OBJSET_STATS.ioctl(z, &mut zc)?;
    let nvbuf = &buf[0..zc.nvlist_dst_size as usize];
    Ok(nvpair::parse(nvbuf)?)
}

fn ioc_dataset_list_next(z: &mut File, dataset: &CStr, cookie: u64) -> Result<(CString, List, u64), Box<dyn Error>> {
    let (mut zc, mut buf) = zc_new();
    let name = dataset.to_bytes_with_nul();
    zc.name[..name.len()].copy_from_slice(&name);
    zc.cookie = cookie;
    sys::ZFS_IOC_DATASET_LIST_NEXT.ioctl(z, &mut zc)?;
    let nvbuf = &buf[0..zc.nvlist_dst_size as usize];
    Ok((CStr::from_bytes_until_nul(&zc.name)?.into(), nvpair::parse(nvbuf)?, zc.cookie))
}

/*
fn print_dataset(dataset: &CStr, stats: Vec<Pair>) -> Result<(), Box<dyn Error>> {
    if let nvpair::PairData::List(ref l) = stats.list.pairs[&CString::new("used")?] {
        if let nvpair::PairData::UInt64(ref u) = l.pairs[&CString::new("value")?] {
            println!("{:?} {:?}", dataset, u);
        }
    }
    Ok(())
}
*/

fn iter_dataset(z: &mut File, dataset: &CStr) -> Result<(), Box<dyn Error>> {
    let mut cookie = 0;
    while let Ok((name, stats, next_cookie)) = ioc_dataset_list_next(z, &dataset, cookie) {
        //print_dataset(&name, stats)?;
        iter_dataset(z, &name)?;
        cookie = next_cookie;
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut z = File::open("/dev/zfs").unwrap_or_else(|_| std::process::exit(0));

	/*
    let (mut zc, mut buf) = zc_new();
    sys::ZFS_IOC_POOL_CONFIGS.ioctl(&mut z, &mut zc).unwrap();
    let nvbuf = &buf[0..zc.nvlist_dst_size as usize];
	*/

	/*
	let props = ioc_pool_get_props(&mut z, &CString::new("lucy").unwrap())?;
	println!("{:#?}", props);
	*/

    let configs = ioc_pool_configs(&mut z)?;
    println!("{:#?}", configs);

    //configs.pairs().for_each(|p| println!("{:?}", p));

    for pool in configs.keys() {
        println!("{:?}", pool);
/*
        let stats = ioc_objset_stats(&mut z, pool)?;
        println!("{:#?}", stats);
        //print_dataset(pool, stats)?;
        iter_dataset(&mut z, pool);

        let props = ioc_pool_get_props(&mut z, pool)?;
        println!("{:#?}", props);
*/
    }

    Ok(())
}
