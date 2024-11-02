mod sequence_space;
mod state;
mod tcpheaderinfo;

use sequence_space::{ReciverSequenceSpace, SenderSequenceSpace};
use state::State;
use tcpheaderinfo::TcpHeaderInfo;

pub struct Connection {
    state: State,
    send: SenderSequenceSpace,
    recv: ReciverSequenceSpace,
    iph: etherparse::Ipv4Header,
    tcpi: TcpHeaderInfo,
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
        let iss = 64;
        let wnd = 10; // TODO: change to something random to follow the RFC
        let mut buf = [0u8; 1500];
        let mut c = Connection {
            state: State::SynRcvd,
            send: SenderSequenceSpace {
                iss,
                una: iss,
                nxt: iss + 1,
                wnd,
                wl1: 0,
                wl2: 0,
                up: false,
            },
            recv: ReciverSequenceSpace {
                irs: tcph.sequence_number(),
                nxt: tcph.sequence_number() + 1,
                wnd: tcph.window_size(),
                up: false,
            },
            iph: etherparse::Ipv4Header::new(
                0,
                64,
                etherparse::IpNumber::TCP,
                iph.destination(),
                iph.source(),
            )
            .unwrap(),
            tcpi: TcpHeaderInfo::new(tcph.destination_port(), tcph.source_port(), iss, wnd),
        };

        // let mut tcph =
        //     etherparse::TcpHeader::new(tcph.destination_port(), tcph.source_port(), iss, wnd);
        let mut tcph_res = c.tcpi.build();

        tcph_res.acknowledgment_number = tcph.sequence_number() + 1;
        tcph_res.syn = true;
        tcph_res.ack = true;
        tcph_res.checksum = tcph_res
            .calc_checksum_ipv4(&c.iph, &[])
            .expect("Failed to cal teh check sum");

        c.iph.set_payload_len(tcph_res.header_len() + 0).unwrap();

        let nbytes = buf.len() - {
            let mut unwritten = &mut buf[..];
            c.iph.write(&mut unwritten).unwrap();
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
        if self.is_ack_valid(tcph.acknowledgment_number()) == false {
            println!(
                "Grey! we got an unvalid ACK number,connection: {:?}, ACK: {}",
                self.send,
                tcph.acknowledgment_number()
            );
            // TODO: check if we are not in a syncronized state yet send rst
            //
            if !self.state.is_state_syncronized() {
                self.send_rst(nic);
            }
            return Ok(());
        }

        let mut seq_len = data.len();
        tcph.syn().then(|| seq_len += 1);
        tcph.fin().then(|| seq_len += 1);
        if self.is_seq_valid(tcph.sequence_number(), seq_len as u32) == false {
            println!(
                "Grey! we got an unvalid SEQ number,\nconnection: {:?},\nSEQ: {},\nSEQ_LEN: {}",
                self.recv,
                tcph.sequence_number(),
                data.len()
            );
            return Ok(());
        }

        match self.state {
            State::SynRcvd => {
                if !tcph.ack() {
                    return Ok(());
                }
                eprintln!("Grey! we estabed a connection");
                self.state = State::Estab;
            }
            State::Estab => {
                println!("{:?}", data.to_ascii_lowercase());
                // unimplemented!();
            }
        }
        Ok(())
    }
    fn send_rst(&self, nic: &mut tun_tap::Iface) {
        // let mut tcph = self.tcph.clone();
        // tcph.rst = true;
    }
    fn is_ack_valid(&self, ack: u32) -> bool {
        if (self.send.una <= self.send.nxt && (ack <= self.send.una || ack > self.send.nxt))
            || (self.send.una > self.send.nxt && (ack <= self.send.una && ack > self.send.nxt))
        {
            return false;
        }
        true
    }

    ///  We have four cases for the acceptability of an incoming segment:
    /// ```
    /// | Segment Length| Receive Window    | Test                                              |
    /// |---------------|-------------------|---------------------------------------------------|
    /// | 0             | 0                 | SEG.SEQ = RCV.NXT                                 |
    /// |---------------|-------------------|---------------------------------------------------|
    /// | 0             | >0                | RCV.NXT =< SEG.SEQ < RCV.NXT+RCV.WND              |
    /// |---------------|-------------------|---------------------------------------------------|
    /// |  0>           | 0                 | not acceptable                                    |
    /// |---------------|-------------------|---------------------------------------------------|
    /// | >0            | >0                | RCV.NXT =< SEG.SEQ < RCV.NXT+RCV.WND              |
    /// |               |                   | or RCV.NXT =< SEG.SEQ+SEG.LEN-1 < RCV.NXT+RCV.WND |
    /// |---------------|-------------------|---------------------------------------------------|
    ///```
    fn is_seq_valid(&self, seq: u32, seq_len: u32) -> bool {
        // case 1 and 3
        if self.recv.wnd == 0 {
            return seq_len == 0 && seq == self.recv.nxt;
        }

        let start = self.recv.nxt;
        let end = start.wrapping_add(self.recv.wnd as u32);
        let last_seq = seq.wrapping_add(seq_len);

        if wrapped_cmp_lt(start.wrapping_sub(1), last_seq, end)
            || wrapped_cmp_lt(start.wrapping_sub(1), last_seq, end)
        {
            return true;
        }
        false
    }
}

/// start < x < end
fn wrapped_cmp_lt<T>(start: T, x: T, end: T) -> bool
where
    T: std::cmp::Ord,
{
    if (start < end && start <= x && x < end) || (end < start && start <= x && x < end) {
        return true;
    }
    false
}
