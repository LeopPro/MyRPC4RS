use std::net::SocketAddr;
use serialization::BincodeSerializer;
use common::Request;
use common::Response;
use std::rc::Rc;
use std::u32;
use std::sync::mpsc;
use std::sync::mpsc::Sender;
use net::Client;
use std::thread;
use std::sync::Arc;
use std::thread::JoinHandle;
use std::mem::swap;

pub struct MyRPCClient {
    socket_addr: SocketAddr,
    request_id: u32,
    sender: Rc<Sender<(Request, Box<FnMut(&Rc<BincodeSerializer>, &Response) + Send>)>>,
    client_thread_handle: Option<JoinHandle<()>>,
}

impl MyRPCClient {
    pub fn new(socket_addr: SocketAddr) -> Self {
        let (sender, receiver) = mpsc::channel();
        let client_thread_handle = thread::spawn(move || {
            let client = Client::new(socket_addr, Rc::new(BincodeSerializer::new()));
            client.start(&Arc::new(receiver));
        });
        Self {
            socket_addr,
            request_id: 0,
            sender: Rc::new(sender),
            client_thread_handle: Some(client_thread_handle),
        }
    }
    pub fn call(&mut self, name: String, params: Vec<Vec<u8>>) -> MyRPCCall {
        let request = Request {
            id: self.request_id,
            name,
            params,
        };
        MyRPCCall { sender: self.sender.clone(), request }
    }
}

impl Drop for MyRPCClient {
    fn drop(&mut self) {
        let req = Request {
            id: u32::MAX,
            name: String::from("stop"),
            params: vec![],
        };
        let callback = |a: &Rc<BincodeSerializer>, b: &Response| { println!("{:?}", b) };
        self.sender.send((req, Box::new(callback))).unwrap();
        let mut client_thread_handle = None;
        swap(&mut self.client_thread_handle, &mut client_thread_handle);
        client_thread_handle.unwrap().join();
    }
}

pub struct MyRPCCall {
    sender: Rc<Sender<(Request, Box<FnMut(&Rc<BincodeSerializer>, &Response) + Send>)>>,
    request: Request,
}

impl MyRPCCall {
    pub fn sync(&self) -> Response {
        unimplemented!()
    }

    pub fn async<F: 'static>(&mut self, callback: F)
        where F: FnMut(&Rc<BincodeSerializer>, &Response) + Send {
        self.sender.send((self.request.clone(), Box::new(callback))).unwrap();
    }
}


#[cfg(test)]
mod tests {
    use client::MyRPCClient;
    use serialization::BincodeSerializer;
    use serialization::Serializer;

    #[test]
    fn client_test() {
        let mut client = MyRPCClient::new("127.0.0.1:8080".parse().unwrap());
        let param1 = BincodeSerializer::new().serialize(&1).unwrap();
        let param2 = BincodeSerializer::new().serialize(&2).unwrap();
        client.call(String::from("test1"), vec![param1, param2]).async(|a,b|{
            println!("{:?}",b)
        })
    }
}