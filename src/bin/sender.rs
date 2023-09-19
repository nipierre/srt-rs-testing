extern crate clap;

use bytes::Bytes;
use clap::{App, Arg};
use futures_util::sink::SinkExt;
use lipsum::lipsum;
use log::info;
use srt_tokio::SrtSocket;
use std::time::{Duration, Instant};
use tokio::time::sleep;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let matches = App::new("srt-sender")
        .version("1.0")
        .about("Listen SRT stream and dump information")
        .arg(
            Arg::with_name("port")
                .short("p")
                .long("port")
                .value_name("PORT")
                .default_value("8999")
                .help("Port")
                .takes_value(true),
        )
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
                .default_value("127.0.0.1")
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
    let port = matches
        .value_of("port")
        .unwrap()
        .parse::<u16>()
        .expect("Unable to parse port-out");
    info!("Value for port: {}", port);

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
        srt_socket
            .send((Instant::now(), Bytes::from(lipsum(1000))))
            .await?;
        sleep(Duration::from_millis(5000)).await;
    }
}
