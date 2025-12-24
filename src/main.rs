#![allow(clippy::unit_arg)]

mod frontend;
mod networking;
mod error;
mod util;

/*
    Plan

    - Database
        - Stores message logs
        - Stores node address of THIS node
        - Stores node addresses of previously messaged clients
        - Associate username with client
    - Networking
        - Receive confirmation for each message
        - Each message must contain the timestamp at which it was sent
    - GUI
*/

fn main() {
    println!("Hello, world!");
}
