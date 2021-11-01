use std::{
    io::{BufReader, BufWriter, Read, Write},
    net::TcpStream,
};

use anyhow::{anyhow, Error, Result};

const HTTP_PORT: u32 = 80;

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

#[derive(Debug, PartialEq)]
pub struct ParsedProxyUrl {
    pub host: String,
    pub port: String,
    pub username: Option<String>,
    pub password: Option<String>,
}

impl ParsedProxyUrl {
    /// new returns a parsed proxy url from given uri
    /// It's forgiven not to start with 'http://'
    /// uri format: http://[username:password@]hostname[:port]/
    pub fn new(uri: &str) -> Result<Self, Error> {
        let host: String;
        let mut port = HTTP_PORT.to_string();
        let mut username: Option<String> = None;
        let mut password: Option<String> = None;
        // skipping 'http://'
        let protocol_pos = uri.find("http://");
        let mut uri = if let Some(pos) = protocol_pos {
            &uri[pos.saturating_add(7)..]
        } else {
            &uri
        };
        // login info parsing
        if let Some(login_info_pos) = uri.find("@") {
            let login_info = &uri[..login_info_pos];
            let username_pos = login_info.find(":");
            if username_pos.is_none() {
                return Err(anyhow!("Supplied login info is malformed: {}", login_info));
            }
            let username_pos = username_pos.unwrap();
            if username_pos == 0 {
                return Err(anyhow!("Expected username in {}", login_info));
            }
            if login_info.len().saturating_sub(1) == username_pos {
                return Err(anyhow!("Expected password in {}", login_info));
            }
            username = Some(String::from(&login_info[..username_pos]));
            password = Some(String::from(&login_info[username_pos.saturating_add(1)..]));

            uri = &uri[login_info_pos.saturating_add(1)..];
        }
        // truncate '/' at the end of uri
        if let Some(slash_pos) = uri.find("/") {
            uri = &uri[..slash_pos];
        }
        // port parsing
        if let Some(colon_pos) = uri.find(":") {
            if colon_pos == uri.len().saturating_sub(1) {
                return Err(anyhow!("Expected port: {}", uri));
            }
            let p = &uri[colon_pos.saturating_add(1)..];
            if p == "0" {
                return Err(anyhow!("Port 0 is invalid: {}", uri));
            }
            host = format!("{}", &uri[..colon_pos]);
            port = format!("{}", p);
        } else {
            host = format!("{}", uri);
        }
        Ok(Self {
            host,
            port,
            username,
            password,
        })
    }
}

pub fn http_get(
    tcp_stream: &TcpStream,
    parsed_url: &ParsedUrl,
    parsed_proxy_url: &Option<ParsedProxyUrl>,
) {
    println!("Retrieving document: '{}'", parsed_url.path);
    let mut reader = BufReader::new(tcp_stream);
    let mut writer = BufWriter::new(tcp_stream);

    // format HTTP request
    let mut header = String::new();
    if let Some(proxy) = parsed_proxy_url {
        header = format!(
            "{}GET http://{}{} HTTP/1.1\r\n",
            header, parsed_url.host, parsed_url.path
        );
        if let (Some(username), Some(password)) = (proxy.username.as_ref(), proxy.password.as_ref())
        {
            let auth = base64::encode(format!("{}:{}", username, password));
            header = format!("{}Proxy-Authorization: BASIC {}\r\n", header, auth);
            header = format!("{}Authorization: BASIC {}\r\n", header, auth);
        }
    } else {
        header = format!("{}GET {} HTTP/1.1\r\n", header, parsed_url.path);
    }
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

    #[test]
    fn test_can_parse_valid_full_proxy_uri() {
        let actual = ParsedProxyUrl::new("http://username:password@example.com:8888/").unwrap();
        let expected = ParsedProxyUrl {
            host: String::from("example.com"),
            port: String::from("8888"),
            username: Some(String::from("username")),
            password: Some(String::from("password")),
        };
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_can_parse_valid_proxy_uri_without_some_part() {
        let actual = ParsedProxyUrl::new("username:password@example.com").unwrap();
        let expected = ParsedProxyUrl {
            host: String::from("example.com"),
            port: String::from(format!("{}", HTTP_PORT)),
            username: Some(String::from("username")),
            password: Some(String::from("password")),
        };
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_can_return_error_username_is_not_supplied() {
        let actual = ParsedProxyUrl::new("http://:password@example.com:8888");
        let expected_err_msg = "Expected username in";
        assert!(actual.is_err(), "ParsedProxyUrl should be error");
        let err_msg = format!("{}", actual.unwrap_err());
        assert!(
            err_msg.contains(expected_err_msg),
            "error message should contain: {}, but actual is: {}",
            expected_err_msg,
            err_msg
        );
    }

    #[test]
    fn test_can_return_error_password_is_not_supplied() {
        let actual = ParsedProxyUrl::new("http://username:@example.com:8888");
        let expected_err_msg = "Expected password in";
        assert!(actual.is_err(), "ParsedProxyUrl should be error");
        let err_msg = format!("{}", actual.unwrap_err());
        assert!(
            err_msg.contains(expected_err_msg),
            "error message should contain: {}, but actual is: {}",
            expected_err_msg,
            err_msg
        );
    }

    #[test]
    fn test_can_return_error_port_is_not_supplied() {
        let actual = ParsedProxyUrl::new("http://username:password@example.com:");
        let expected_err_msg = "Expected port";
        assert!(actual.is_err(), "ParsedProxyUrl should be error");
        let err_msg = format!("{}", actual.unwrap_err());
        assert!(
            err_msg.contains(expected_err_msg),
            "error message should contain: {}, but actual is: {}",
            expected_err_msg,
            err_msg
        );
    }

    #[test]
    fn test_can_return_error_invalid_port() {
        let actual = ParsedProxyUrl::new("http://username:password@example.com:0");
        let expected_err_msg = "Port 0 is invalid";
        assert!(actual.is_err(), "ParsedProxyUrl should be error");
        let err_msg = format!("{}", actual.unwrap_err());
        assert!(
            err_msg.contains(expected_err_msg),
            "error message should contain: {}, but actual is: {}",
            expected_err_msg,
            err_msg
        );
    }
}
