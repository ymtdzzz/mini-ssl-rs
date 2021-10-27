use std::env;

use anyhow::{anyhow, Result};
use mini_ssl_rs::app::{App, Client, Server};

const ARGS_ERROR_MSG: &str =
    "\n Usage (as client): mini-ssl-rs client <URL>\n Usage (as server): mini-ssl-rs server";

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    // arguments validation
    if args.len() < 2 {
        return Err(anyhow!(ARGS_ERROR_MSG));
    }

    match &*args[1] {
        "client" => {
            let mut client = Client::new(&args)?;
            client.run()?;
        }
        "server" => {
            let mut server = Server::new();
            server.run()?;
        }
        _ => {
            return Err(anyhow!(ARGS_ERROR_MSG));
        }
    }

    Ok(())
}
