pub enum State {
    // Closed,
    // Listen,
    SynRcvd,
    Estab,
}

///  non-synchronized state (LISTEN, SYN-SENT, SYN-RECEIVED)
impl State {
    pub fn is_state_syncronized(&self) -> bool {
        match self {
            State::SynRcvd => false,
            State::Estab => true,
        }
    }
}
