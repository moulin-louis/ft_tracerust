use std::error::Error;
use std::net::{ToSocketAddrs, UdpSocket};
use std::ops::Add;
use std::thread;
use std::time::Duration;

use rand::Rng;

#[derive(Debug, Copy, Clone, Default)]
struct UdpPacketHeader {
    //source port of the packet
    src_port: u16,
    //dest port of the packet. Range from 33434 to 33534.
    dest_port: u16,
    //len of the header + metadata udp packet (min  8)
    len: u16,
    //Used for error-checking of the header and data. This field is optional in IPv4, and mandatory in most cases in IPv6.The field carries all-zeros if unused.
    //Will be unused for this project since we will only use IPv4
    checksum: u16,
}

impl UdpPacketHeader {
    unsafe fn to_bytes(&self) -> &[u8] {
        std::slice::from_raw_parts((self as *const UdpPacketHeader) as *const u8, std::mem::size_of::<IPv4Header>())
    }
}

#[derive(Debug, Copy, Clone, Default)]
struct IPv4Header {
    //version: 4 bits => Always equal to 4 + ipv4 header len.
    //ihl: 4 bits => The IHL field contains the size of the IPv4 header; it has 4 bits that specify the number of 32-bit words in the header.
    version_ihl: u8,
    // dscp: 6 bits => Specifies differentiated services (DiffServ).
    // ecm:  2 bits => Allows end-to-end notification of network congestion without dropping packets
    dscp_ecn: u8,
    //Defines the entire packet size in bytes, including header and data. Minimum is 20 bytes.
    total_len: u16,
    //Used for uniquely identifying the group of fragments of a single IP datagram.
    id: u16,
    //flags: 3 bits => bit 0: Reserved; must be zero, bit 1: Don't Fragment (DF), bit 2: More Fragments (MF)
    //fragment_offset: 13 bits => Specifies the offset of a particular fragment relative to the beginning of the original unfragmented IP datagram.
    flags_fragment_offset: u32,
    //Limits a datagram's lifetime to prevent network failure in the event of a routing loop.
    ttl: u8,
    //Defines the protocol used in the data portion of the IP datagram.
    protocol: u8,
    //Used for error checking of the header.
    header_checksum: u16,
    //Sender of the packet.
    src_addr: u32,
    //Receiver of the packet.
    dest_addr: u32,
}

impl IPv4Header {
    unsafe fn to_bytes(&self) -> &[u8] {
        std::slice::from_raw_parts((self as *const IPv4Header) as *const u8, std::mem::size_of::<IPv4Header>())
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    if std::env::args().len() != 2 {
        eprintln!("please provide an hostname");
        return Ok(());
    }
    let mut dest = std::env::args().collect::<Vec<String>>()[1].clone();

    let port_offset = rand::thread_rng().gen_range(1..=100);
    let port = 34254 + port_offset;

    let dest_addr = dest.add(":1").to_socket_addrs()?.nth(0).unwrap();
    println!("dest addr = [{dest_addr}]");

    let socket = UdpSocket::bind("0.0.0.0:34254")?;
    socket.connect(dest_addr)?;
    println!("socket connected to {dest_addr}");

    let mut ipv4_header = IPv4Header {
        version_ihl: 4 << 4 | 5,
        dscp_ecn: 0,
        total_len: 20,
        id: 0,
        flags_fragment_offset: 0,
        ttl: 0, //wil be incremented during loop,
        protocol: 0x11,
        header_checksum: 0, //need to be computed righ afeter
        src_addr: 0,
        dest_addr: 0,
    };
    // todo!("implement ipv4 checksum computation")
    println!("ipv4 header = {:?}", ipv4_header);
    let upd_packet_header = UdpPacketHeader {
        src_port: 4243,
        dest_port: port,
        len: 8,
        checksum: 0,
    };
    for _ in 0..255 {
        ipv4_header.ttl += 1;
        println!("ttl = {}", ipv4_header.ttl);
        unsafe {
            let mut buff = ipv4_header.to_bytes().to_vec();
            buff.extend(upd_packet_header.to_bytes());
            socket.send(ipv4_header.to_bytes()).expect(format!("send failed, ipv4 header: {:?}, udp header = {:?}", ipv4_header, upd_packet_header).as_str());
            println!("one packet sended");
            thread::sleep(Duration::from_millis(100));
        }
    }
    return Ok(());
}
