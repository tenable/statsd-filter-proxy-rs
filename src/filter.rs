use bytes::{BufMut, Bytes, BytesMut};
use std::str;

pub fn filter(block_list: &Vec<String>, buf: &[u8]) -> String {
    let statsd_str = unsafe { str::from_utf8_unchecked(&buf) };

    let result_itr = statsd_str.split("\n").filter(|line| {
        for prefix in block_list.iter() {
            if line.starts_with(prefix) {
                return false;
            }
        }
        return true;
    });

    let result = result_itr.collect::<Vec<&str>>().join("\n");

    return result;
}

pub fn filter_2(block_list: &[Bytes], data: &[u8]) -> Bytes {
    let mut buffer = BytesMut::with_capacity(data.len());

    'outer: for line in data.split(|x| x == &b'\n') {
        for prefix in block_list {
            if line.starts_with(prefix) {
                continue 'outer;
            }
        }

        buffer.put(line);
    }

    buffer.freeze()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_not_block_multi_metric() {
        let block_list = vec![String::from("notfoo"), String::from("otherfoo")];
        let statsd_str_bytes = "foo:1|c\nfoo:2|c\nfoo:3|c".as_bytes();
        let result = filter(&block_list, &statsd_str_bytes);
        assert_eq!("foo:1|c\nfoo:2|c\nfoo:3|c", result);
    }

    #[test]
    fn test_should_not_block_single_metric() {
        let block_list = vec![String::from("notfoo"), String::from("otherfoo")];
        let statsd_str_bytes = "foo:1|c".as_bytes();
        let result = filter(&block_list, &statsd_str_bytes);
        assert_eq!("foo:1|c", result);
    }

    #[test]
    fn test_should_block_completely_single_metric() {
        let block_list = vec![String::from("foo"), String::from("otherfoo")];
        let statsd_str_bytes = "foo:1|c".as_bytes();
        let result = filter(&block_list, &statsd_str_bytes);
        assert_eq!("", result);
    }

    #[test]
    fn test_should_block_completely_multi_metric() {
        let block_list = vec![String::from("foo"), String::from("otherfoo")];
        let statsd_str_bytes = "foo:1|c\nfoo:2|c\nfoo:3|c".as_bytes();
        let result = filter(&block_list, &statsd_str_bytes);
        assert_eq!("", result);
    }

    #[test]
    fn test_should_block_partially_multi_metric() {
        let block_list = vec![String::from("foo"), String::from("otherfoo")];
        let statsd_str_bytes = "notfoo:1|c\nfoo:2|c\nnotfoo:3|c".as_bytes();
        let result = filter(&block_list, &statsd_str_bytes);
        assert_eq!("notfoo:1|c\nnotfoo:3|c", result);
    }
}
