use std::error::Error;

pub type DynStdError = Box<dyn Error>;
pub type ResultDynError<T> = Result<T, DynStdError>;
