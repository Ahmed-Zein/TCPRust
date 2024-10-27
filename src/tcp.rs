pub enum State {
    // Closed,
    // Listen,
    SynRcvd,
    Estab,
}
pub struct Connection {
    state: State,
    send: SenderSequenceSpace,
    recv: ReciverSequenceSpace,
}

struct SenderSequenceSpace {
    /// send unacknowledged
    una: u32,
    /// send next
    nxt: u32,
    /// send window
    wnd: u16,
    /// send urgent pointer
    up: bool,
    /// segment sequence number used for last window update
    wl1: usize,
    /// segment acknowledgment number used for last window update
    wl2: usize,
    /// - initial send sequence number
    iss: u32,
}
struct ReciverSequenceSpace {
    /// receive next
    nxt: u32,
    /// receive window
    wnd: u16,
    /// receive urgent pointer
    up: bool,
    /// initial receive sequence number
    irs: u32,
}

impl Connection {
    pub fn accept(
        nic: &mut tun_tap::Iface,
        iph: etherparse::Ipv4HeaderSlice,
        tcph: etherparse::TcpHeaderSlice,
        data: &[u8],
    ) -> Result<Option<Connection>, String> {
        if !tcph.syn() {
            return Err("syn not exist".to_string());
        }
        let iss = 10;
        let mut buf = [0u8; 1500];
        let c = Connection {
            state: State::SynRcvd,
            send: SenderSequenceSpace {
                iss,
                una: iss,
                nxt: iss + 1,
                wnd: 10, // TODO: change to something random to follow the RFC
                wl1: 0,
                wl2: 0,
                up: false,
            },
            recv: ReciverSequenceSpace {
                nxt: tcph.sequence_number() + 1,
                wnd: tcph.window_size(),
                up: false,
                irs: tcph.sequence_number(),
            },
        };

        // we build up a tcp header
        let mut tcph_res = etherparse::TcpHeader::new(
            tcph.destination_port(),
            tcph.source_port(),
            c.send.nxt,
            c.send.wnd,
        );

        let iph_res = etherparse::Ipv4Header::new(
            tcph_res.header_len().try_into().unwrap(),
            64,
            etherparse::IpNumber::TCP,
            iph.destination(),
            iph.source(),
        )
        .unwrap();

        tcph_res.acknowledgment_number = tcph.sequence_number() + 1;
        tcph_res.syn = true;
        tcph_res.ack = true;
        tcph_res.checksum = tcph_res
            .calc_checksum_ipv4(&iph_res, &[])
            .expect("Failed to cal teh check sum");

        let nbytes = buf.len() - {
            let mut unwritten = &mut buf[..];
            iph_res.write(&mut unwritten).unwrap();
            tcph_res.write(&mut unwritten).unwrap();
            unwritten.len()
        };

        let _ = nic.send(&buf[..nbytes]);
        Ok(Some(c))
    }
    pub fn on_packet(
        &mut self,
        nic: &mut tun_tap::Iface,
        iph: etherparse::Ipv4HeaderSlice,
        tcph: etherparse::TcpHeaderSlice,
        data: &[u8],
    ) -> std::io::Result<()> {
        match self.state {
            State::SynRcvd => {
                if tcph.ack() && tcph.sequence_number() == self.recv.nxt {
                    eprintln!("Grey! we estabed a connection");
                    self.state = State::Estab;
                }
            }
            State::Estab => {
                unimplemented!();
            }
        }
        Ok(())
    }
}
