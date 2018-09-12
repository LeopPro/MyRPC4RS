use std::collections::HashMap;
use bytes::Bytes;
use error::Error;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Request {
    id: u32,
    name: String,
    params: Vec<Vec<u8>>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Response {
    id: u32,
    name: String,
    result: Result<Vec<u8>, Error>,
}

#[cfg(test)]
mod tests {
    #[test]
    fn command_test() {}
}