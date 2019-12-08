use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use std::{env, thread};

use tokio::{net, runtime, task};

const THROUGHPUT_PERIOD: u64 = 5;

async fn run(port_from: usize, port_to: usize) {
    let counter = Arc::new(AtomicU64::new(0));

    // Start one task per port
    for port in port_from..port_to {
        let counter_port = counter.clone();
        task::spawn(async move {
            let addr = format!("127.0.0.1:{}", port);
            let mut socket = net::UdpSocket::bind(addr).await.unwrap();
            let mut buf = vec![0; 160];
            while let Ok(_) = socket.recv(&mut buf[..]).await {
                counter_port.fetch_add(1, Ordering::SeqCst);
            }
        });
    }

    task::spawn(async move {
        let mut last = Instant::now();
        loop {
            tokio::time::delay_for(Duration::from_secs(THROUGHPUT_PERIOD)).await;
            let now = Instant::now();
            let delta = now - last;
            println!(
                "{} / s",
                counter.swap(0, Ordering::SeqCst) / delta.as_secs()
            );
            last = now;
        }
    });

    // Block forever
    let () = futures::future::pending().await;
}

fn main() {
    let args = env::args().collect::<Vec<_>>();
    assert_eq!(args.len(), 4);
    let n_streams: usize = args[1].parse().unwrap();
    let n_groups: Option<usize> = args[2].parse().ok();
    let max_throttling: Option<u64> = args[3].parse().ok();

    let mut threads = Vec::with_capacity(n_groups.unwrap_or(1));
    if let Some(n_groups) = n_groups {
        let mut port_start = 40000;
        for _ in 0..n_groups {
            let mut builder = runtime::Builder::new();
            if let Some(max_throttling) = max_throttling {
                println!("Throttling up to {} ms", max_throttling);
                builder.max_throttling(Duration::from_millis(max_throttling));
            }

            let mut runtime = builder
                .enable_io()
                .enable_time()
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
