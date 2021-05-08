use bytes::BytesMut;
use statsd_filter_proxy_rs::{old_run_server, run_server, Config};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::net::UdpSocket;
use tokio::sync::Notify;
use tokio::time::{self, error::Elapsed};

#[tokio::main]
async fn main() {
    let msg_count = 1_000;
    let threads = 4;

    let config = Config {
        listen_host: String::from("127.0.0.1"),
        listen_port: 8125,
        target_host: String::from("127.0.0.1"),
        target_port: 8126,
        metric_blocklist: vec![
            String::from("foo"),
            String::from("otherfoo"),
            String::from("otherfoo2"),
            String::from("otherfoo3"),
            String::from("otherfoo4"),
            String::from("otherfoo5"),
            String::from("otherfoo6"),
            String::from("otherfoo7"),
            String::from("otherfoo8"),
            String::from("otherfoo9"),
        ],
        multi_thread: None,
    };

    let stop = Arc::new(Notify::new());

    tokio::spawn(spawn_old(config.clone(), false, stop.clone()));

    let mut received = 0;
    match run(msg_count, threads, &mut received).await {
        Ok(duration) => {
            println!(
                "[Filter classic] Processed {} messages in {:?} | {:?}/msg",
                received,
                duration,
                duration / received as u32
            );
        }
        Err(_) => {
            println!(
                "[Filter classic] Test timed out after 60s and {} messages",
                received
            );
        }
    }

    stop.notify_waiters();

    // Cooldown for cleanup
    time::sleep(Duration::from_secs(10)).await;

    let stop = Arc::new(Notify::new());

    tokio::spawn(spawn_old(config.clone(), true, stop.clone()));

    let mut received = 0;
    match run(msg_count, threads, &mut received).await {
        Ok(duration) => {
            println!(
                "[Filter 2] Processed {} messages in {:?} | {:?}/msg",
                received,
                duration,
                duration / received as u32
            );
        }
        Err(_) => {
            println!(
                "[Filter 2] Test timed out after 60s and {} messages",
                received
            );
        }
    }

    stop.notify_waiters();

    // Cooldown for cleanup
    time::sleep(Duration::from_secs(10)).await;

    let stop = Arc::new(Notify::new());

    tokio::spawn(spawn_new(config, stop.clone()));

    let mut received = 0;
    match run(msg_count, threads, &mut received).await {
        Ok(duration) => {
            println!(
                "[Codec] Processed {} messages in {:?} | {:?}/msg",
                received,
                duration,
                duration / received as u32
            );
        }
        Err(_) => {
            println!("[Codec] Test timed out after 60s and {} messages", received);
        }
    }

    stop.notify_waiters();

    // Cooldown for cleanup
    time::sleep(Duration::from_secs(10)).await;
}

async fn spawn_new(config: Config, stop: Arc<Notify>) {
    tokio::select! {
        _ = run_server(config) => {}
        _ = stop.notified() => {}
    }
}

async fn spawn_old(config: Config, use_fn_2: bool, stop: Arc<Notify>) {
    tokio::select! {
        _ = old_run_server(config, use_fn_2) => {}
        _ = stop.notified() => {}
    }
}

async fn run(msg_count: usize, threads: usize, received: &mut usize) -> Result<Duration, Elapsed> {
    let notify = Arc::new(Notify::new());

    for i in 0..threads {
        let notify = notify.clone();
        let addr = format!("127.0.0.1:1000{}", i);
        let sock = UdpSocket::bind(addr).await.unwrap();
        sock.connect("127.0.0.1:8125").await.unwrap();

        tokio::spawn(async move {
            notify.notified().await;

            for i in 0..msg_count {
                let _ = sock.send(format!("metric:{}|c\n", i).as_bytes()).await;
            }
        });
    }

    let expected = msg_count * threads;

    time::timeout(Duration::from_secs(10), receive(notify, expected, received)).await
}

async fn receive(notify: Arc<Notify>, expected: usize, received: &mut usize) -> Duration {
    let sock = UdpSocket::bind("127.0.0.1:8126").await.unwrap();
    let mut buffer = BytesMut::with_capacity(10_000_000);

    let start = Instant::now();

    notify.notify_waiters();

    while let Ok((_, _)) = sock.recv_from(&mut buffer).await {
        *received += buffer[..].split(|x| x == &b'\n').count();

        buffer.clear();

        if *received >= expected {
            break;
        }
    }

    start.elapsed()
}
