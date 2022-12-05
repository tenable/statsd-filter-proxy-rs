use bytes::{BufMut, Bytes, BytesMut};
use tokio_util::codec::Decoder;

pub struct FilteredCodec {
    pub block_list: Vec<Bytes>,
}

impl Decoder for FilteredCodec {
    type Item = BytesMut;
    type Error = std::io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let data = match src.iter().rposition(|c| c == &b'\n') {
            Some(pos) => src.split_to(pos + 1),
            None => return Ok(None),
        };

        let mut buffer = BytesMut::with_capacity(data.len());

        'outer: for line in data[..].split(|x| x == &b'\n') {
            if line.is_empty() {
                continue;
            }

            for prefix in &self.block_list {
                if line.starts_with(prefix) {
                    continue 'outer;
                }
            }

            buffer.put(line);
            buffer.put_u8(b'\n');
        }

        Ok(if buffer.is_empty() {
            None
        } else {
            Some(buffer)
        })
    }
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
