#[derive(Debug)]
pub struct SenderSequenceSpace {
    /// The oldest unacknowledged sequence number, which is the beginning of the range of bytes sent but not yet acknowledged by the receiver.
    pub una: u32,
    /// The next sequence number to be sent. This marks the end of the range of bytes that have been sent out.
    pub nxt: u32,
    /// send window
    pub wnd: u16,
    /// send urgent pointer
    pub up: bool,
    /// segment sequence number used for last window update
    pub wl1: usize,
    /// segment acknowledgment number used for last window update
    pub wl2: usize,
    /// - initial send sequence number
    pub iss: u32,
}

#[derive(Debug)]
pub struct ReciverSequenceSpace {
    /// receive next
    pub nxt: u32,
    /// receive window
    pub wnd: u16,
    /// receive urgent pointer
    pub up: bool,
    /// initial receive sequence number
    pub irs: u32,
}
