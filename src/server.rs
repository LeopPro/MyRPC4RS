use std::net::SocketAddr;
use serialization::Serializer;
use serialization::BincodeSerializer;
use std::collections::HashMap;
use bytes::Bytes;
use std::error;
use serde::Serialize;
use serde::Deserialize;
use bincode::deserialize;
use std::io;
use error::Result;
use error::Error;
use net::Server;

pub struct Processes {
    function_map: HashMap<String, Box<FnMut(&Vec<Bytes>) -> Result<Bytes>>>
}

impl Processes {
    pub fn new() -> Self {
        Self {
            function_map: HashMap::new(),
        }
    }

    fn get_function(&mut self, name: &str) -> Option<&mut Box<FnMut(&Vec<Bytes>) -> Result<Bytes>>> {
        self.function_map.get_mut(name)
    }

    pub fn insert_function<F: 'static>(&mut self, name: String, function: F)
        where F: FnMut(&Vec<Bytes>) -> Result<Bytes> {
        self.function_map.insert(name, Box::new(function));
    }

    pub fn execute_function(&mut self, name: &str, params: &Vec<Bytes>) -> Result<Bytes> {
        match self.get_function(name) {
            Some(function) => function(params),
            None => Err(Error::FunctionNotFound)
        }
    }
}

pub struct MyRPCServer {
    socket_addr: SocketAddr,
    serializer: BincodeSerializer,
    processes: Processes,
//    server:Server,
}

impl MyRPCServer {
    pub fn new(socket_addr: SocketAddr) -> Self {
        Self {
            socket_addr,
            serializer: BincodeSerializer::new(),
            processes: Processes::new(),
        }
    }

    pub fn register_function<F: 'static>(&mut self, name: String, function: F)
        where F: FnMut(&Vec<Bytes>) -> Result<Bytes> {
        self.processes.insert_function(name, function);
    }

    pub fn start_server(&mut self){

    }
}


#[cfg(test)]
mod tests {
    use server::Processes;
    use serialization::Serializer;
    use serialization::BincodeSerializer;
    use bytes::Bytes;
    use server::MyRPCServer;
    use std::borrow::BorrowMut;

    #[test]
    fn process_test() {
        let mut processse = Processes::new();
        processse.insert_function(String::from("test"), |process| {
            let param1: u32 = BincodeSerializer::new().deserialize(&process[0]).unwrap();
            let param2: u32 = BincodeSerializer::new().deserialize(&process[1]).unwrap();
            println!("{}", param1);
            println!("{}", param2);
            Ok(Bytes::from("123"))
        });


        let param1 = BincodeSerializer::new().serialize(&1).unwrap();
        let param2 = BincodeSerializer::new().serialize(&2).unwrap();
        let result = processse.execute_function("test", &vec![param1, param2]);
        println!("{:?}", result);
    }

    #[test]
    fn myrpcserver_test() {}
}