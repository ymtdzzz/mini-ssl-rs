use std::{
    io::{BufReader, BufWriter, Read, Write},
    net::TcpStream,
};

#[derive(Debug, PartialEq)]
pub struct ParsedUrl {
    pub host: String,
    pub path: String,
}

impl ParsedUrl {
    /// new returns a parsed url from given uri
    pub fn new(uri: &str) -> Option<Self> {
        let mut uri = uri.to_string();
        if uri.chars().last()? != '/' {
            uri = format!("{0}{1}", uri, "/");
        }

        let host_start_pos = uri.find("//")?.saturating_add(2);
        let host_and_path = &uri[host_start_pos..];
        let path_start_pos = host_and_path.find("/")?.saturating_add(host_start_pos);
        let host = &uri[host_start_pos..path_start_pos];
        let path = &uri[path_start_pos..];

        Some(Self {
            host: String::from(host),
            path: String::from(path),
        })
    }
}

pub fn http_get(tcp_stream: &TcpStream, parsed_url: &ParsedUrl) {
    println!("Retrieving document: '{}'", parsed_url.path);
    let mut reader = BufReader::new(tcp_stream);
    let mut writer = BufWriter::new(tcp_stream);

    // format HTTP request
    let header = format!("GET {} HTTP/1.1\r\n", parsed_url.path);
    let header = format!(
        "{}HOST: {}\r\nConnection: close\r\n\r\n",
        header, parsed_url.host
    );
    println!("GET request sending...");
    println!("-- Request --\n{}", header);

    tcp_write(&mut writer, &header);
    print!("{}", tcp_read(&mut reader));
}

fn tcp_read(reader: &mut BufReader<&TcpStream>) -> String {
    let mut msg = String::new();
    reader
        .read_to_string(&mut msg)
        .expect("Failed to read lines from tcp stream");
    msg
}

fn tcp_write(writer: &mut BufWriter<&TcpStream>, msg: &str) {
    writer
        .write(msg.as_bytes())
        .expect("Failed to send message to tcp stream");
    writer.flush().unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_can_parse_valid_uri() {
        let actual = ParsedUrl::new("http://www.example.com/this/is/path");
        let expected = Some(ParsedUrl {
            host: String::from("www.example.com"),
            path: String::from("/this/is/path/"),
        });
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_can_parse_valid_uri_without_path() {
        let actual = ParsedUrl::new("http://www.example.com/");
        let expected = Some(ParsedUrl {
            host: String::from("www.example.com"),
            path: String::from("/"),
        });
        assert_eq!(actual, expected);
        let actual = ParsedUrl::new("http://www.example.com");
        let expected = Some(ParsedUrl {
            host: String::from("www.example.com"),
            path: String::from("/"),
        });
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_can_return_none_with_invalid_uri() {
        let result = ParsedUrl::new("thisisinvaliduri.com");
        assert!(result.is_none());
    }
}
