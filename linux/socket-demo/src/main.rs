//! # Rust Socket Demo for P3411 Wireless Embedded / P3642 Cerberust
//!
//! Copyright (c) 2018, Cambridge Consultants Ltd.
//! See the top-level README.md for licence details.
//!
//! This crate is a binary which exchanges messages with an IPU using an
//! AF_RPMSG socket. The IPU must be running the ipu-demo-rs firmware from
//! https://gitlab.uk.cambridgeconsultants.com/p3411/cerberus/ipu-demo-rs

// ****************************************************************************
//
// Crates
//
// ****************************************************************************

extern crate libc;
extern crate socket2;

// ****************************************************************************
//
// Imports
//
// ****************************************************************************

use libc::{sa_family_t, socklen_t};

// ****************************************************************************
//
// Sub-modules
//
// ****************************************************************************

// None

// ****************************************************************************
//
// Macros
//
// ****************************************************************************

// None

// ****************************************************************************
//
// Public Types / Traits
//
// ****************************************************************************

// None

// ****************************************************************************
//
// Public Data
//
// ****************************************************************************

// None

// ****************************************************************************
//
// Private Types / Traits
//
// ****************************************************************************

#[repr(C)]
pub struct SockAddrRpMsg {
    pub sa_family: sa_family_t,
    pub vproc_id: u32,
    pub addr: u32,
}

#[repr(C)]
pub struct TestMessage {
    pub test1: u32,
    pub test2: u32,
    pub test3: u32,
}

// ****************************************************************************
//
// Private Data
//
// ****************************************************************************

const AF_RPMSG: i32 = 43;
const CORE_ID: u32 = 0;
const HOST_ID: u32 = 100;
const REMOTE_ID: u32 = 61;
const CONNECT_TIMEOUT_SECONDS: u64 = 3;
const READ_TIMEOUT_SECONDS: u64 = 2;

// ****************************************************************************
//
// Public Functions
//
// ****************************************************************************

fn main() {
    let (mut tx, mut rx) = sockets();
    println!("Running...");
    let mut sent_count = 0;
    let mut recv_count = 0;
    const TOTAL_COUNT: usize = 10_000;
    for i in 0..TOTAL_COUNT {
        if send(&mut tx) {
            sent_count += 1;
        }
        if recv(&mut rx) {
            recv_count += 1;
        }
        if ((i + 1) % (TOTAL_COUNT / 10)) == 0 {
            println!("{}: Sent: {}, Recv: {}", i, sent_count, recv_count);
        }
    }
    println!("Final Total: Sent: {}, Recv: {}", sent_count, recv_count);
    println!("Test complete.");
}

// ****************************************************************************
//
// Private Functions
//
// ****************************************************************************

fn sockets() -> (socket2::Socket, socket2::Socket) {
    let tx = socket2::Socket::new(AF_RPMSG.into(), socket2::Type::seqpacket(), None)
        .expect("socket() failed");
    let addr = SockAddrRpMsg {
        sa_family: AF_RPMSG as u16,
        vproc_id: CORE_ID,
        addr: REMOTE_ID,
    };
    tx.connect_timeout(
        &addr.into(),
        ::std::time::Duration::new(CONNECT_TIMEOUT_SECONDS, 0),
    )
    .expect("connect_timeout() failed");
    tx.set_nonblocking(false).expect("set_nonblocking() failed");

    let rx = socket2::Socket::new(AF_RPMSG.into(), socket2::Type::seqpacket(), None)
        .expect("socket() failed");
    let addr = SockAddrRpMsg {
        sa_family: AF_RPMSG as u16,
        vproc_id: CORE_ID,
        addr: HOST_ID,
    };
    rx.bind(&addr.into()).expect("bind() failed");
    rx.set_nonblocking(false).expect("set_nonblocking() failed");
    rx.set_read_timeout(Some(::std::time::Duration::new(READ_TIMEOUT_SECONDS, 0)))
        .expect("read_timeout() failed");

    (tx, rx)
}

fn send(tx: &mut socket2::Socket) -> bool {
    let msg = TestMessage {
        test1: 0xAAAAAAAA,
        test2: 0xBBBBBBBB,
        test3: 0xCCCCCCCC,
    };
    let slice = msg.as_bytes();
    match tx.send(&slice) {
        Ok(_) => true,
        Err(e) => {
            eprintln!("Got send error: {:?}", e);
            false
        }
    }
}

fn recv(rx: &mut socket2::Socket) -> bool {
    let mut buffer = [0u8; 512];
    loop {
        match rx.recv_from(&mut buffer) {
            Err(e) => {
                if e.raw_os_error() == Some(107) {
                    println!("Waiting for socket...");
                } else {
                    eprintln!("Error reading: {:?}", e);
                    return false;
                }
            }
            Ok((len, _addr)) => {
                let valid = &buffer[0..len];
                let _message = std::str::from_utf8(valid).expect("Invalid UTF-8 received");
                return len == 64;
            }
        }
    }
}

impl<'a> TestMessage {
    fn as_bytes(self) -> [u8; 12] {
        unsafe { std::mem::transmute::<TestMessage, [u8; 12]>(self) }
    }
}

impl std::convert::From<SockAddrRpMsg> for socket2::SockAddr {
    fn from(addr: SockAddrRpMsg) -> socket2::SockAddr {
        unsafe {
            let ptr = &addr as *const _ as *const _;
            socket2::SockAddr::from_raw_parts(
                ptr,
                std::mem::size_of::<SockAddrRpMsg>() as socklen_t,
            )
        }
    }
}

// ****************************************************************************
//
// End Of File
//
// ****************************************************************************
