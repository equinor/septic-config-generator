use serde::Serialize;
use std::collections::HashMap;
pub mod args;
pub mod config;
pub mod datasource;
pub mod renderer;

pub type DataSourceRow = Vec<(String, HashMap<String, CtxDataType>)>;

#[derive(Debug)]

pub enum CtxErrorType {
    /// Division by 0 error
    Div0,
    /// Unavailable value error
    NA,
    /// Invalid name error
    Name,
    /// Null value error
    Null,
    /// Number error
    Num,
    /// Invalid cell reference error
    Ref,
    /// Value error
    Value,
    /// Getting data
    GettingData,
}

#[derive(Debug)]
pub enum CtxDataType {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    DateTime(f64),
    Error(CtxErrorType),
    Empty,
}

impl Serialize for CtxDataType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            CtxDataType::Int(value) => serializer.serialize_i64(*value),
            CtxDataType::Float(value) => serializer.serialize_f64(*value),
            CtxDataType::String(value) => serializer.serialize_str(value),
            CtxDataType::Bool(value) => serializer.serialize_bool(*value),
            CtxDataType::DateTime(value) => serializer.serialize_f64(*value),
            CtxDataType::Error(value) => {
                let s = match value {
                    CtxErrorType::Div0 => "#DIV/0!",
                    CtxErrorType::NA => "#N/A",
                    CtxErrorType::Name => "#NAME?",
                    CtxErrorType::Null => "#NULL!",
                    CtxErrorType::Num => "#NUM!",
                    CtxErrorType::Ref => "#REF!",
                    CtxErrorType::Value => "#VALUE!",
                    _ => "#UNKNOWN!",
                };
                serializer.serialize_str(s)
            }

            // DataTypeSer::Error(_) => serializer.serialize_str("Error in cell"), // Do I need to handle this as Err or just return a special value?
            CtxDataType::Empty => serializer.serialize_unit(),
        }
    }
}
