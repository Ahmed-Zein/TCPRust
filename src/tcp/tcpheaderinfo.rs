pub struct TcpHeaderInfo {
    src: u16,
    dst: u16,
    wnd: u16,
    seq_number: u32,
}

impl TcpHeaderInfo {
    pub fn new(src: u16, dst: u16, seq_number: u32, wnd: u16) -> TcpHeaderInfo {
        Self {
            src,
            dst,
            wnd,
            seq_number,
        }
    }
    pub fn build(&self) -> etherparse::TcpHeader {
        etherparse::TcpHeader::new(self.src, self.dst, self.seq_number, self.wnd)
    }
}
