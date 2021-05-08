mod config;
pub mod filter;
pub mod filtered_codec;
mod server;

pub use self::config::Config;
pub use self::server::run_server;
use bytes::Bytes;
use log::{debug, info};
use std::net::SocketAddr;
use tokio::net::UdpSocket;

pub async fn old_run_server(config: Config, use_fn_2: bool) -> std::io::Result<()> {
    let addr = format!("{}:{}", config.listen_host, config.listen_port);
    let sock = UdpSocket::bind(addr).await?;

    let block_list = config.metric_blocklist;
    let block_list_2 = block_list
        .iter()
        .cloned()
        .map(Bytes::from)
        .collect::<Vec<_>>();

    info!("Listening on: {}", sock.local_addr()?);

    let mut buf = [0; 8192];

    let target_addr: SocketAddr = format!("{}:{}", config.target_host, config.target_port)
        .parse()
        .expect("Unable to parse socket address");

    loop {
        let (len, addr) = sock.recv_from(&mut buf).await?;
        debug!("{:?} bytes received from {:?} onto {:p}", len, addr, &buf);

        let len = if use_fn_2 {
            let filtered = crate::filter::filter_2(&block_list_2, &buf);

            if filtered.is_empty() {
                continue;
            }

            sock.send_to(&filtered[..], &target_addr).await.unwrap()
        } else {
            let filtered = crate::filter::filter(&block_list, &buf);

            if filtered.is_empty() {
                continue;
            }

            sock.send_to(filtered.as_bytes(), &target_addr)
                .await
                .unwrap()
        };

        debug!("Echoed {} bytes to {}", len, target_addr);
    }
}
