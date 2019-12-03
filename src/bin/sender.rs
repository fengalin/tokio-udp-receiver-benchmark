use std::net;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::{env, thread, time};

fn main() {
    let args = env::args().collect::<Vec<_>>();
    assert_eq!(args.len(), 2);
    let n_streams: u16 = args[1].parse().unwrap();

    let buffer = [0; 160];
    let socket = net::UdpSocket::bind("0.0.0.0:0").unwrap();

    let ipaddr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
    let destinations = (40000..(40000 + n_streams))
        .map(|port| SocketAddr::new(ipaddr, port))
        .collect::<Vec<_>>();

    let wait = time::Duration::from_millis(20);

    thread::sleep(time::Duration::from_millis(1000));

    loop {
        let now = time::Instant::now();

        for dest in &destinations {
            socket.send_to(&buffer, dest).unwrap();
        }

        let elapsed = now.elapsed();
        if elapsed < wait {
            thread::sleep(wait - elapsed);
        }
    }
}
