use crate::config::Config;
use crate::filtered_codec::FilteredCodec;
use bytes::Bytes;
use futures_util::StreamExt;
use log::{debug, info, warn};
use std::net::SocketAddr;
use tokio::net::UdpSocket;
use tokio_util::udp::UdpFramed;

pub async fn run_server(config: Config) -> std::io::Result<()> {
    let addr = format!("{}:{}", config.listen_host, config.listen_port);
    let sock = UdpSocket::bind(addr).await?;

    info!("Listening on: {}", sock.local_addr()?);

    let target_addr: SocketAddr = format!("{}:{}", config.target_host, config.target_port)
        .parse()
        .expect("Unable to parse socket address");

    let codec = FilteredCodec {
        block_list: config
            .metric_blocklist
            .into_iter()
            .map(Bytes::from)
            .collect(),
    };

    let mut framed = UdpFramed::new(sock, codec);

    while let Some(frame) = framed.next().await {
        let frame = match frame {
            Ok((frame, addr)) => {
                debug!(
                    "Received frame {:?} from {:?}",
                    std::str::from_utf8(&frame),
                    addr
                );

                frame
            }
            Err(e) => {
                warn!("Failed to parse frame: {:?}", e);

                continue;
            }
        };

        if let Err(e) = framed.get_ref().send_to(&frame, &target_addr).await {
            warn!("Couldn't send metric to metrics server: {:?}", e);
        }
    }

    Ok(())
}
