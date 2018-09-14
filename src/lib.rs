#[macro_use]
extern crate futures;
extern crate tokio_core;
extern crate tokio_io;
extern crate bytes;
extern crate byteorder;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate bincode;

mod net;
mod common;
mod serialization;

#[macro_use]
pub mod server;
pub mod client;
mod error;


#[cfg(test)]
mod tests {

    #[test]
    fn start_server() {
    }
}