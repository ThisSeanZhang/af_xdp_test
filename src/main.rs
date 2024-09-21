use libbpf_rs::{MapCore, MapFlags};

use xsk_rs::{
    config::{LibxdpFlags, UmemConfig},
    socket::Socket,
    umem::Umem,
};

use libbpf_rs::skel::{OpenSkel, SkelBuilder};
use std::{mem::MaybeUninit, os::fd::AsRawFd};

pub mod xdp_test {
    include!(concat!(env!("OUT_DIR"), "/xdp_test.skel.rs"));
}

use xdp_test::*;

use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread, time,
};

fn main() {
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .unwrap();

    let (dev1_umem, mut dev1_descs) =
        Umem::new(UmemConfig::default(), 32.try_into().unwrap(), false)
            .expect("failed to create UMEM");

    let config = xsk_rs::config::SocketConfigBuilder::new()
        .libxdp_flags(LibxdpFlags::XSK_LIBXDP_FLAGS_INHIBIT_PROG_LOAD)
        .build();
    let (mut dev1_tx_q, mut dev1_rx_q, dev1_fq_and_cq) =
        unsafe { Socket::new(config, &dev1_umem, &"ens4".parse().unwrap(), 0) }
            .expect("failed to create dev1 socket");

    let mut landscape_builder = XdpTestSkelBuilder::default();
    landscape_builder.obj_builder.debug(true);

    let mut open_object = MaybeUninit::uninit();

    let landscape_open = landscape_builder.open(&mut open_object).unwrap();
    let mut landscape_skel = landscape_open.load().unwrap();

    let _link = landscape_skel.progs.xdp_pass.attach_xdp(3).unwrap();
    let map_fd = landscape_skel.maps.xsks_map;

    let key = 0 as u32;
    let fd = dev1_tx_q.fd().as_raw_fd() as u32;

    let (mut dev1_fq, _dev1_cq) = dev1_fq_and_cq.expect("missing dev2 fill queue and comp queue");
    unsafe {
        dev1_fq.produce(&dev1_descs);
    }
    map_fd
        .update(&key.to_le_bytes(), &fd.to_le_bytes(), MapFlags::ANY)
        .unwrap();
    // unsafe {
    //     let ret = libbpf_sys::bpf_map_update_elem(xsk_map_fd, &key as *const _ as *const _, &fd as *const _ as *const _, 0);
    //     if ret != 0 {
    //         panic!("Failed to update xskmap: {}", ret);
    //     }
    // }

    while running.load(Ordering::SeqCst) {
        println!("loop step.");
        let pkts_recvd = unsafe { dev1_rx_q.poll_and_consume(&mut dev1_descs, 100).unwrap() };

        for recv_desc in dev1_descs.iter().take(pkts_recvd) {
            let data = unsafe { dev1_umem.data(recv_desc) };
            println!("Received data: {:?}", data.contents());
        }

        if pkts_recvd == 0 && dev1_tx_q.needs_wakeup() {
            println!("to weak up");
            let fd = dev1_tx_q.fd_mut();
            // let result = dev1_fq.wakeup(fd, 100);
            unsafe { dev1_fq.produce_and_wakeup(&dev1_descs, fd, 100).unwrap() };
            // println!("to weak up: {result:?}");
        }
        thread::sleep(time::Duration::from_secs(1));
    }
}
