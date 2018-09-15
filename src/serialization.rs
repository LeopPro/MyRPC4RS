use bincode::serialize as bincode_serialize;
use bincode::deserialize as bincode_deserialize;
use std::error;
use serde::Serialize;
use serde::Deserialize;

type Error = Box<error::Error>;
type Result<T> = ::std::result::Result<T, Error>;

pub trait Serializer {
    fn serialize<T: ?Sized>(&self, value: &T) -> Result<Vec<u8>>
        where T: Serialize;
    fn deserialize<'a, T>(&self, bytes: &'a [u8]) -> Result<T>
        where T: Deserialize<'a>;
}
#[derive(Clone)]
pub struct BincodeSerializer;

impl BincodeSerializer {
    pub fn new() -> Self {
        Self{}
    }
}

impl Serializer for BincodeSerializer {
    fn serialize<T: ?Sized>(&self, value: &T) -> Result<Vec<u8>>
        where T: Serialize {
        Ok(Vec::from(bincode_serialize(value)?))
    }

    fn deserialize<'a, T>(&self, bytes: &'a [u8]) -> Result<T>
        where T: Deserialize<'a>{
        Ok(bincode_deserialize(bytes)?)
    }

}