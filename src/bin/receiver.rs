use std::{env, thread};

use tokio::prelude::*;
use tokio::{net, runtime, task};

async fn run(port_from: usize, port_to: usize) {
    // Start one task per port
    for port in port_from..port_to {
        task::spawn(async move {
            let addr = format!("127.0.0.1:{}", port);
            let mut socket = net::UdpSocket::bind(addr).await.unwrap();
            let mut buf = vec![0; 160];
            while let Ok(_) = socket.recv(&mut buf[..]).await {}
        });
    }

    // Block forever
    let () = futures::future::pending().await;
}

fn main() {
    let args = env::args().collect::<Vec<_>>();
    assert_eq!(args.len(), 3);
    let n_streams: usize = args[1].parse().unwrap();
    let n_groups: Option<usize> = args[2].parse().ok();

    let mut threads = Vec::with_capacity(n_groups.unwrap_or(1));
    if let Some(n_groups) = n_groups {
        let mut port_start = 40000;
        for _ in 0..n_groups {
            let mut runtime = runtime::Builder::new()
                .enable_io()
                .basic_scheduler()
                .build()
                .unwrap();

            let thread = thread::spawn(move || {
                runtime.block_on(run(port_start, port_start + n_streams / n_groups));
            });
            port_start += n_streams / n_groups;

            threads.push(thread);
        }
    } else {
        let mut runtime = runtime::Runtime::new().unwrap();

        let thread = thread::spawn(move || {
            runtime.block_on(run(40000, 40000 + n_streams));
        });

        threads.push(thread);
    }

    for thread in threads {
        thread.join().unwrap();
    }
}
