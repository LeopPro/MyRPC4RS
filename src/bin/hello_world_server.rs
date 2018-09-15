#[macro_use]
extern crate myrpc4rs;

use myrpc4rs::server::MyRPCServer;
use myrpc4rs::serialization::Serializer;

fn main(){
    let mut myrpc = MyRPCServer::new("127.0.0.1:6180".parse().unwrap());
    myrpc_function!(myrpc,test1,param1<u32>,param2<u32>,{
            println!("{},{}",param1,param2);
            String::from(format!("hello world!a+b={}",param1+param2))
        });
    myrpc.start_server();
}