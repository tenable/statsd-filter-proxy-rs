mod config;
mod filtered_codec;
mod server;

use std::env;
use std::path::Path;

#[tokio::main]
async fn main() {
    env_logger::init();
    let config_path_env = env::var("PROXY_CONFIG_FILE").expect("PROXY_CONFIG_FILE must be set");

    let path = Path::new(&config_path_env);
    let config = config::parse(&path);

    server::run_server(config)
        .await
        .expect("Unable to run server");
}
