use bincode::serialize as bincode_serialize;
use bincode::deserialize as bincode_deserialize;
use std::error;
use serde::Serialize;
use serde::Deserialize;
use bytes::Bytes;

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


#[cfg(test)]
mod tests {
    use serialization::BincodeSerializer;
    use serialization::Serializer;
    use serde::Deserialize;
    use bytes::Bytes;
    use bincode::deserialize as bincode_deserialize;
    use std::error;

    type Error = Box<error::Error>;
    type Result<T> = ::std::result::Result<T, Error>;

    #[test]
    fn serializer_test() {
//        let mut message = Message::new(0, String::from("test_fun"));
//        message.add_param(Value::Integer(123));
//        message.add_param(Value::String(String::from("321")));
//        let message_clone = message.clone();
//        let bytes = BincodeSerializer.serialize(&message);
//        assert!(bytes.is_ok());
//        let message_copy = BincodeSerializer.deserialize(&bytes.unwrap());
//        assert!(message_copy.is_ok());
//        assert_eq!(message_clone, message_copy.unwrap());

    }

}