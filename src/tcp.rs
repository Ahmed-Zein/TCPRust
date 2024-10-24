pub struct State {}

impl Default for State {
    fn default() -> Self {
        State {}
    }
}

impl State {
    pub fn on_packet(
        &self,
        iph: etherparse::Ipv4HeaderSlice,
        tcph: etherparse::TcpHeaderSlice,
        data: &[u8],
    ) {
        eprintln!(
            "{}:{} -> {}:{}, with {}byte TCP",
            iph.source_addr(),
            tcph.source_port(),
            iph.destination_addr(),
            tcph.destination_port(),
            data.len()
        )
    }
}
