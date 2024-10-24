use tun_tap::Iface;

fn main() -> Result<(), ()> {
    let nic = Iface::new("tap-0", tun_tap::Mode::Tun).expect("failed to create a new TUN Devive");
    let mut buf = [0u8; 1504];
    nic.recv(&mut buf);
    Ok(())
}
