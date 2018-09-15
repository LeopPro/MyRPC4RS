#[macro_use]
extern crate myrpc4rs;

use myrpc4rs::serialization::Serializer;
use myrpc4rs::client::MyRPCClient;
use myrpc4rs::error::Error;

fn main(){
    let mut client = MyRPCClient::new("127.0.0.1:6180".parse().unwrap());
    myrpc_call_async!(client,test1,&8,&4;aa <String>{
            println!("{:?}",aa);
    });

    let resp = myrpc_call_sync!(client,test1,&16,&24;<String>);
    println!("{:?}", resp);
}