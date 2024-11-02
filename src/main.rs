mod tcp;
use etherparse::{IpNumber, Ipv4HeaderSlice, TcpHeaderSlice};
use std::collections::HashMap;
use std::net::Ipv4Addr;
use tun_tap::Iface;

const _IPV4_PROTO: u16 = 0x0800;
const TCP_PROTO: u8 = 0x0006;
const TAP_NAME: &str = "tun0";

/*
 * first we get the IPV4 packet and extract the TCP packet from it
 */

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
struct Quad {
    src: (Ipv4Addr, u16),
    dst: (Ipv4Addr, u16),
}

fn main() -> Result<(), ()> {
    let mut nic = Iface::without_packet_info(TAP_NAME, tun_tap::Mode::Tun)
        .expect("Grey we failed to create a new TUN Devive");
    let mut buf = [0u8; 1504];

    let mut connections: HashMap<Quad, tcp::Connection> = Default::default();
    loop {
        let nbytes = nic.recv(&mut buf).unwrap();
        // let _flags = u16::from_be_bytes([buf[0], buf[1]]);
        //let eth_proto = u16::from_be_bytes([buf[2], buf[3]]);
        // if eth_proto != IPV4_PROTO {
        //     continue;
        // }

        match Ipv4HeaderSlice::from_slice(&buf[..nbytes]) {
            Ok(iph) => {
                if iph.protocol() != IpNumber(TCP_PROTO) {
                    continue;
                }
                match TcpHeaderSlice::from_slice(&buf[iph.slice().len()..nbytes]) {
                    Ok(tcph) => {
                        use std::collections::hash_map::Entry;
                        let datai = iph.slice().len() + tcph.slice().len();

                        match connections.entry(Quad {
                            src: (iph.source_addr(), tcph.source_port()),
                            dst: (iph.destination_addr(), tcph.destination_port()),
                        }) {
                            Entry::Vacant(v) => {
                                if let Some(c) = tcp::Connection::accept(
                                    &mut nic,
                                    iph,
                                    tcph,
                                    &buf[datai..nbytes],
                                )
                                .unwrap()
                                {
                                    v.insert(c);
                                };
                            }
                            Entry::Occupied(mut c) => {
                                let _ =
                                    c.get_mut()
                                        .on_packet(&mut nic, iph, tcph, &buf[datai..nbytes]);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("{}", e);
                    }
                }
            }
            Err(_e) => {
                // eprintln!("Weird Packet {}", e);
            }
        }
    }
}
