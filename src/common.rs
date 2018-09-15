use error::Error;
use std::u32;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Request {
    pub id: u32,
    pub name: String,
    pub params: Vec<Vec<u8>>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Response {
    pub id: u32,
    pub name: String,
    pub result: Result<Vec<u8>, Error>,
}

impl Response {
    pub fn err(request: Request, error: Error) -> Self {
        Self {
            id: request.id,
            name: request.name,
            result: Err(error),
        }
    }
    pub fn err_unknow_request(error: Error) -> Self {
        Self {
            id: u32::MAX,
            name: String::new(),
            result: Err(error),
        }
    }
    pub fn from(request: Request, result: Vec<u8>) -> Self {
        Self {
            id: request.id,
            name: request.name,
            result: Ok(result),
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn command_test() {}
}