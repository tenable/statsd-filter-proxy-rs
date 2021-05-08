use log::trace;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    pub listen_host: String,
    pub listen_port: u16,
    pub target_host: String,
    pub target_port: u16,
    pub metric_blocklist: Vec<String>,
    pub multi_thread: Option<bool>,
}

pub fn parse(config_path: &Path) -> Config {
    let contents = fs::read_to_string(config_path).expect("Unable to load configuration file");

    trace!("{}", contents);

    let config: Config =
        serde_json::from_str(&contents).expect("Unable to decode configuration file");

    trace!("{:?}", config.metric_blocklist);

    config
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_parse_config() {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("test/good_config.json");

        let config = parse(d.as_path());

        assert_eq!("0.0.0.0", config.listen_host);
        assert_eq!(8125, config.listen_port);
        assert_eq!("127.0.0.1", config.target_host);
        assert_eq!(8126, config.target_port);
        assert_eq!(
            &vec![
                String::from("metrics1"),
                String::from("metrics2"),
                String::from("metrics3")
            ],
            &config.metric_blocklist
        );
    }

    #[test]
    #[should_panic(expected = "Unable to decode configuration file")]
    fn test_parse_config_bad_json() {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("test/bad_config.json");

        parse(d.as_path());
    }

    #[test]
    #[should_panic(expected = "Unable to load configuration file")]
    fn test_parse_config_missing_file() {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("test/no_such_file.json");

        parse(d.as_path());
    }
}
