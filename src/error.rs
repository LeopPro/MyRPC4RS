
pub type Result<T> = ::std::result::Result<T, Error>;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub enum Error {
    FunctionNotFound,
    ParamDeserializeFail,

}
