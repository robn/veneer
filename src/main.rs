mod ioctls;
mod nvlist;

use std::error::Error;
//use std::ffi::CString;

fn main() -> Result<(), Box<dyn Error>> {
    let mut file = std::fs::File::open("/dev/zfs").unwrap_or_else(|_| std::process::exit(0));

    {
        let mut zc: ioctls::Command = Default::default();
        let buf: [u8; 262144] = [0; 262144];
        zc.nvlist_dst = buf.as_ptr();
        zc.nvlist_dst_size = 262144;
        ioctls::ZFS_IOC_POOL_CONFIGS.ioctl(&mut file, &mut zc).unwrap();
        let nvbuf = &buf[0..zc.nvlist_dst_size as usize];
        let nv = nvlist::unpack(nvbuf)?;
        println!("{:#?}", nv);
    }

    /*
    {
        let mut zc: ioctls::Command = Default::default();
        let name = CString::new("lucy").unwrap().into_bytes_with_nul();
        zc.name[..name.len()].copy_from_slice(&name);
        let buf: [u8; 262144] = [0; 262144];
        zc.nvlist_dst = buf.as_ptr();
        zc.nvlist_dst_size = 262144;
        ioctls::ZFS_IOC_POOL_STATS.ioctl(&mut file, &mut zc).unwrap();
        let nvbuf = &buf[0..zc.nvlist_dst_size as usize];
        hexdump::hexdump(&nvbuf);
        let nv = nvlist::unpack(nvbuf)?;
        println!("{:#?}", nv);
    }
    */

    Ok(())
}
