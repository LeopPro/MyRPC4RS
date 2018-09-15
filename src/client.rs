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
    request_id: u32,
    sender: Rc<Sender<(Request, Box<FnMut(&Rc<BincodeSerializer>, &Response) + Send>)>>,
    client_thread_handle: Option<JoinHandle<()>>,
    serializer: BincodeSerializer,
}

impl MyRPCClient {
    pub fn new(socket_addr: SocketAddr) -> Self {
        let (sender, receiver) = mpsc::channel();
        let client_thread_handle = thread::spawn(move || {
            let client = Client::new(socket_addr, Rc::new(BincodeSerializer::new()));
            client.start(&Arc::new(receiver));
        });
        Self {
            request_id: 0,
            sender: Rc::new(sender),
            client_thread_handle: Some(client_thread_handle),
            serializer: BincodeSerializer::new(),
        }
    }
    pub fn call(&mut self, name: String, params: Vec<Vec<u8>>) -> MyRPCCall {
        let request = Request {
            id: self.request_id,
            name,
            params,
        };
        self.request_id = (self.request_id + 1) % (u32::MAX - 1);
        MyRPCCall { sender: self.sender.clone(), request }
    }

    pub fn get_serializer(&self) -> &BincodeSerializer {
        &self.serializer
    }
}

impl Drop for MyRPCClient {
    fn drop(&mut self) {
        let req = Request {
            id: u32::MAX,
            name: String::from("stop"),
            params: vec![],
        };
        let callback = |_: &Rc<BincodeSerializer>, _: &Response| {};
        self.sender.send((req, Box::new(callback))).unwrap();
        let mut client_thread_handle = None;
        swap(&mut self.client_thread_handle, &mut client_thread_handle);
        client_thread_handle.unwrap().join().unwrap();
    }
}

pub struct MyRPCCall {
    sender: Rc<Sender<(Request, Box<FnMut(&Rc<BincodeSerializer>, &Response) + Send>)>>,
    request: Request,
}

impl MyRPCCall {
    pub fn sync(&self) -> Response {
        let (sender, receiver) = mpsc::channel();
        let callback = move |_: &Rc<BincodeSerializer>, resp: &Response| {
            sender.send(resp.clone()).unwrap();
        };
        self.sender.send((self.request.clone(), Box::new(callback))).unwrap();
        return receiver.recv().unwrap();
    }

    pub fn async<F: 'static>(&mut self, callback: F)
        where F: FnMut(&Rc<BincodeSerializer>, &Response) + Send {
        self.sender.send((self.request.clone(), Box::new(callback))).unwrap();
    }
}

#[macro_export]
macro_rules! myrpc_call_async {
    ($myrpc_client:expr, $function_name:expr, $($param:expr),+ ; $response:ident<$t:ty> $myrpc_block:block) => {

        let mut params = Vec::new();
        $(params.push($myrpc_client.get_serializer().serialize($param).unwrap());)+
        $myrpc_client.call(String::from(stringify!($function_name)), params).async(|serializer,response|{
            let $response:Result<$t,&Error> = match &response.result {
                Ok(bytes)=>Ok(serializer.deserialize(&bytes).unwrap()),
                Err(error)=>Err(error),
            };
            $myrpc_block;
        })
    }
}

#[macro_export]
macro_rules! myrpc_call_sync {
    ($myrpc_client:expr, $function_name:expr, $($param:expr),+ ;<$t:ty>) => {
        {
            let mut params = Vec::new();
            $(params.push($myrpc_client.get_serializer().serialize($param).unwrap());)+
            let response = $myrpc_client.call(String::from(stringify!($function_name)), params).sync();
            let result:Result<$t,Error> = match response.result {
                Ok(bytes)=>Ok($myrpc_client.get_serializer().deserialize(&bytes).unwrap()),
                Err(error)=>Err(error),
            };
            result
        }
    }
}
#[cfg(test)]
mod tests {
    use client::MyRPCClient;
    use serialization::Serializer;
    use error::Error;

    #[test]
    fn client_test() {
        let mut client = MyRPCClient::new("127.0.0.1:8080".parse().unwrap());
//        let param1 = client.get_serializer().serialize(&1).unwrap();
//        let param2 = client.get_serializer().serialize(&2).unwrap();
//        let a = client.call(String::from("test1"), vec![param1, param2]).sync();
        myrpc_call_async!(client,test1,&8,&4;aa <u32>{
            println!("{:?}",aa);
        });

        let resp = myrpc_call_sync!(client,test1,&16,&24;<u32>);
        println!("{:?}", resp);
    }
}