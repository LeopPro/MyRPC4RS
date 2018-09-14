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
use std::cell::RefCell;
use common::Request;
use std::rc::Rc;


pub struct Processes {
    function_map: RefCell<HashMap<String, Box<FnMut(&Rc<BincodeSerializer>, &Vec<Vec<u8>>) -> Result<Vec<u8>>>>>,
    serializer: Rc<BincodeSerializer>,
}

impl Processes {
    pub fn new(serializer: Rc<BincodeSerializer>) -> Self {
        Self {
            function_map: RefCell::new(HashMap::new()),
            serializer,
        }
    }

    pub fn insert_function<F: 'static>(&self, name: String, function: F)
        where F: FnMut(&Rc<BincodeSerializer>, &Vec<Vec<u8>>) -> Result<Vec<u8>> {
        self.function_map.borrow_mut().insert(name, Box::new(function));
    }

    pub fn execute_function(&self, name: &str, params: &Vec<Vec<u8>>) -> Result<Vec<u8>> {
        let mut function = self.function_map.borrow_mut();
        let function = function.get_mut(name);
        let result = match function {
            Some(function) => function(&self.serializer, params),
            None => Err(Error::FunctionNotFound)
        };
        result
    }

    pub fn get_serializer(&self) -> &BincodeSerializer {
        &self.serializer
    }
}

pub struct MyRPCServer {
    socket_addr: SocketAddr,
    serializer: BincodeSerializer,
    processes: Rc<Processes>,
    server: Server,
}

impl MyRPCServer {
    pub fn new(socket_addr: SocketAddr) -> Self {
        Self {
            socket_addr,
            serializer: BincodeSerializer::new(),
            processes: Rc::new(Processes::new(Rc::new(BincodeSerializer::new()))),
            server: Server::new(socket_addr),
        }
    }

    pub fn register_function<F: 'static>(&self, name: String, function: F)
        where F: FnMut(&Rc<BincodeSerializer>, &Vec<Vec<u8>>) -> Result<Vec<u8>> {
        self.processes.insert_function(name, function);
    }

    pub fn start_server(&mut self) {
        self.server.start(self.processes.clone())
    }

    pub fn get_serializer(&self) -> &BincodeSerializer {
        &self.serializer
    }
}

#[macro_export]
macro_rules! myrpc_function {
    ($myrpc_server:expr, $function_name:expr, $($param:ident<$t:ty>),+ , $myrpc_block:block) => {
        $myrpc_server.register_function(String::from(stringify!($function_name)), |serializer, process| {
            let mut i = 0;
            $(let $param:$t = serializer.deserialize(&process[i]).unwrap();i+=1;)+
            Ok(serializer.serialize(&$myrpc_block).unwrap())
        });
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
    use std::rc::Rc;

    #[test]
    fn process_test() {
        let mut processse = Processes::new(Rc::new(BincodeSerializer::new()));
        processse.insert_function(String::from("test"), |serializer, process| {
            let param1: u32 = serializer.deserialize(&process[0]).unwrap();
            let param2: u32 = serializer.deserialize(&process[1]).unwrap();
            assert_eq!(1, param1);
            assert_eq!(2, param2);
            let a = serializer.serialize(&(param1+param2)).unwrap();
            Ok(a)
        });


        let param1 = BincodeSerializer::new().serialize(&1).unwrap();
        let param2 = BincodeSerializer::new().serialize(&2).unwrap();
        let result = processse.execute_function("test", &vec![param1, param2]);
        assert_eq!(Ok(vec![1, 2, 3]), result)
    }

    #[test]
    fn myrpcserver_test() {
        let mut myrpc = MyRPCServer::new("127.0.0.1:8080".parse().unwrap());
        myrpc_function!(myrpc,test1,param1<u32>,param2<u32>,{
            println!("{},{}",param1,param2);
            param1+param2
        });
        myrpc.start_server();
    }
}