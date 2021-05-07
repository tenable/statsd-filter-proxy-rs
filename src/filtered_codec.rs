use bytes::{BufMut, Bytes, BytesMut};
use std::collections::BTreeSet;
use tokio_util::codec::Decoder;

pub struct FilteredCodec {
    pub block_list: BTreeSet<Bytes>,
}

impl Decoder for FilteredCodec {
    type Item = BytesMut;
    type Error = std::io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let mut buffer = BytesMut::new();

        while let Some(datagram) = get_datagram(src) {
            let tag = match datagram.iter().position(|x| x == &b':') {
                Some(pos) => &datagram[..pos],
                None => &datagram[..],
            };

            if self.block_list.contains(tag) {
                continue;
            }

            buffer.put(datagram);
        }

        Ok(if buffer.is_empty() {
            None
        } else {
            Some(buffer)
        })
    }
}

fn get_datagram(src: &mut BytesMut) -> Option<BytesMut> {
    src.iter()
        .position(|c| c == &b'\n')
        .map(|pos| src.split_to(pos + 1))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_not_block_multi_metric() {
        let block_list = vec![String::from("notfoo"), String::from("otherfoo")];
        let mut filter = FilteredCodec {
            block_list: block_list.into_iter().map(Bytes::from).collect(),
        };

        let statsd_str_bytes = "foo:1|c\nfoo:2|c\nfoo:3|c\n".as_bytes();
        let result = filter.decode(&mut BytesMut::from(statsd_str_bytes));
        let result = result.unwrap().unwrap();
        assert_eq!(&statsd_str_bytes[..], result);
    }

    #[test]
    fn test_should_not_block_single_metric() {
        let block_list = vec![String::from("notfoo"), String::from("otherfoo")];
        let mut filter = FilteredCodec {
            block_list: block_list.into_iter().map(Bytes::from).collect(),
        };

        let statsd_str_bytes = "foo:1|c\n".as_bytes();
        let result = filter.decode(&mut BytesMut::from(statsd_str_bytes));
        let result = result.unwrap().unwrap();
        assert_eq!(&statsd_str_bytes[..], result);
    }

    #[test]
    fn test_should_block_completely_single_metric() {
        let block_list = vec![String::from("foo"), String::from("otherfoo")];
        let mut filter = FilteredCodec {
            block_list: block_list.into_iter().map(Bytes::from).collect(),
        };

        let statsd_str_bytes = "foo:1|c\n".as_bytes();
        let result = filter.decode(&mut BytesMut::from(statsd_str_bytes));
        let result = result.unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_should_block_completely_multi_metric() {
        let block_list = vec![String::from("foo"), String::from("otherfoo")];
        let mut filter = FilteredCodec {
            block_list: block_list.into_iter().map(Bytes::from).collect(),
        };

        let statsd_str_bytes = "foo:1|c\nfoo:2|c\nfoo:3|c\n".as_bytes();
        let result = filter.decode(&mut BytesMut::from(statsd_str_bytes));
        let result = result.unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_should_block_partially_multi_metric() {
        let block_list = vec![String::from("foo"), String::from("otherfoo")];
        let mut filter = FilteredCodec {
            block_list: block_list.into_iter().map(Bytes::from).collect(),
        };

        let statsd_str_bytes = "notfoo:1|c\nfoo:2|c\nnotfoo:3|c\n".as_bytes();
        let result = filter.decode(&mut BytesMut::from(statsd_str_bytes));
        let result = result.unwrap().unwrap();
        assert_eq!("notfoo:1|c\nnotfoo:3|c\n".as_bytes(), result);
    }
}
