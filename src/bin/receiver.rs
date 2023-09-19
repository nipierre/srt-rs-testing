use clap::{App, Arg};
use futures_util::TryStreamExt;
use log::{debug, error, info};
use srt_tokio::SrtSocket;
use std::io::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let matches = App::new("srt-listener")
        .version("1.0")
        .about("Listen SRT stream and dump information")
        .arg(
            Arg::with_name("listener")
                .short("l")
                .long("listener")
                .value_name("listener")
                .help("Listener mode")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("url")
                .short("u")
                .long("url")
                .value_name("URL")
                .help("URL")
                .default_value("localhost")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("port")
                .short("p")
                .long("port")
                .value_name("PORT")
                .help("Port")
                .default_value("8999")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("v")
                .short("v")
                .multiple(true)
                .help("Sets the level of verbosity"),
        )
        .get_matches();

    match matches.occurrences_of("v") {
        0 => simple_logger::init_with_level(log::Level::Error).unwrap(),
        1 => simple_logger::init_with_level(log::Level::Info).unwrap(),
        2 => simple_logger::init_with_level(log::Level::Debug).unwrap(),
        _ => simple_logger::init_with_level(log::Level::Trace).unwrap(),
    }

    let listener = matches.is_present("listener");
    let port = matches.value_of("port").unwrap().parse::<u16>().unwrap();

    loop {
        let mut srt_socket = if listener {
            info!("Wait for connection..");
            SrtSocket::builder().listen_on(port).await?
        } else {
            let url = matches.value_of("url").unwrap().parse::<String>().unwrap();
            let uri = format!("{}:{}", url, port);
            info!("Try connection on {}", uri);
            SrtSocket::builder().call(&uri[..], None).await?
        };

        info!("Stream hooked");

        loop {
            match srt_socket.try_next().await {
                Ok(Some((instant, bytes))) => {
                    debug!("Packet instant: {:?}", instant);
                    debug!("Packet data: {:?}", &bytes);

                    let string_content = std::str::from_utf8(&bytes[..])
                        .expect("Cannot convert bytes to string")
                        .to_string();

                    info!("{}", string_content);
                }
                Ok(None) => {
                    debug!("Packet without data, connection timeout ?");
                }
                Err(message) => {
                    error!("{:?}", message);
                }
            }
        }
    }
}
