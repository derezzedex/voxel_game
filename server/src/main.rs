use std::io::stdin;
use anyhow::{Result, Error};
use std::thread;
use std::time::Instant;

use laminar::{Packet, Socket, SocketEvent};
use bincode::{serialize, deserialize};
use serde::{Serialize, Deserialize};

const SERVER: &str = "127.0.0.1:12345";
const CLIENT: &str = "127.0.0.1:12346";

#[derive(Debug, Serialize, Deserialize)]
pub enum Package{
    ConnectionStart,
    Input(Vec<u8>),
    ConnectionEnd,
}

fn main() -> Result<(), Error> {
    println!("Starting server..");
    let mut socket = Socket::bind(SERVER)?;
    let (sender, receiver) = (socket.get_packet_sender(), socket.get_event_receiver());
    let _thread = thread::spawn(move || socket.start_polling());

    let timer = Instant::now();
    let mut position = 0.;
    let velocity = 1.;

    loop {
        position += velocity;

        if let Ok(event) = receiver.recv() {
            match event {
                SocketEvent::Packet(packet) => {
                    let ip = packet.addr().ip();

                    let content = deserialize::<Package>(packet.payload()).expect("couldn't deserialize");
                    // println!("Received {:?} from {:?}", content, ip);
                    match content{
                        Package::ConnectionStart => (),
                        Package::Input(data) => (),
                        Package::ConnectionEnd => (),
                        _ => (),
                    }

                }
                SocketEvent::Timeout(address) => {
                    println!("Client timed out: {}", address);
                }
                _ => {}
            }
        }

        let message = serialize(&position).expect("Coulnd't serialize position");
        sender
            .send(Packet::reliable_unordered(
                CLIENT.parse().unwrap(),
                message,
            ))
            .expect("This should send");
    }

    Ok(())
}
