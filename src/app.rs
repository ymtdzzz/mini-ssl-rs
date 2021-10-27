use std::net::{TcpStream, ToSocketAddrs};

use anyhow::{anyhow, Context, Result};

use crate::http::{http_get, ParsedUrl};

const HTTP_PORT: u32 = 80;

pub trait App {
    fn run(&mut self) -> Result<()>;
}

pub struct Client {
    parsed_url: ParsedUrl,
}

impl Client {
    pub fn new(args: &Vec<String>) -> Result<Self> {
        let idx = 2;

        let parsed_url = ParsedUrl::new(&args[idx]);
        if parsed_url.is_none() {
            return Err(anyhow!("Error - malformed URL '{}'", args[idx]));
        }

        Ok(Self {
            parsed_url: parsed_url.unwrap(),
        })
    }
}

impl App for Client {
    fn run(&mut self) -> Result<()> {
        println!("Connecting to host {}", self.parsed_url.host);

        let addrs = format!("{}:{}", self.parsed_url.host, HTTP_PORT).to_socket_addrs();

        println!("Resolved IP: {:?}", addrs);

        if addrs.is_err() {
            return Err(anyhow!("Error in name resolution"));
        }

        let mut addrs = addrs.unwrap();

        if let Some(addr) = addrs.find(|x| (*x).is_ipv4()) {
            let stream = TcpStream::connect(addr).with_context(|| "Unable to connect to host.")?;
            http_get(&stream, &self.parsed_url);
        } else {
            return Err(anyhow!("Invalid Host:Port combination."));
        }

        Ok(())
    }
}

pub struct Server;

impl Server {
    pub fn new() -> Self {
        Self {}
    }
}

impl App for Server {
    fn run(&mut self) -> Result<()> {
        // TODO
        Ok(())
    }
}
