use etherparse::{IpNumber, Ipv4HeaderSlice, TcpHeaderSlice};
use std::collections::HashMap;
use std::net::Ipv4Addr;
use tun_tap::Iface;

mod tcp;

const IPV4_PROTO: u16 = 0x0800;
const TCP_PROTO: u8 = 0x0006;
const TAP_NAME: &str = "tap0";

/*
 * first we get the IPV4 packet and extract the TCP packet from it
 */

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
struct Quad {
    src: (Ipv4Addr, u16),
    dst: (Ipv4Addr, u16),
}

fn main() -> Result<(), ()> {
    let nic = Iface::new(TAP_NAME, tun_tap::Mode::Tun)
        .expect("Grey we failed to create a new TUN Devive");
    let mut buf = [0u8; 1504];

    let mut connections: HashMap<Quad, tcp::State> = Default::default();
    loop {
        let nbytes = nic.recv(&mut buf).unwrap();
        let _flags = u16::from_be_bytes([buf[0], buf[1]]);
        let eth_proto = u16::from_be_bytes([buf[2], buf[3]]);

        if eth_proto != IPV4_PROTO {
            continue;
        }

        match Ipv4HeaderSlice::from_slice(&buf[4..nbytes]) {
            Ok(iph) => {
                if iph.protocol() != IpNumber(TCP_PROTO) {
                    continue;
                }
                match TcpHeaderSlice::from_slice(&buf[4 + iph.slice().len()..nbytes]) {
                    Ok(tcph) => {
                        let state = connections
                            .entry(Quad {
                                src: (iph.source_addr(), tcph.source_port()),
                                dst: (iph.destination_addr(), tcph.destination_port()),
                            })
                            .or_insert(tcp::State {});
                        let datai = 4 + iph.slice().len() + tcph.slice().len();
                        state.on_packet(iph, tcph, &buf[datai..nbytes]);
                    }
                    Err(e) => {
                        eprintln!("{}", e);
                    }
                }
            }
            Err(e) => {
                eprintln!("Weird Packet {}", e);
            }
        }
    }
}
