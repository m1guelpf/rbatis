use crate::decode::Decoder::Decoder;
use std::sync::Arc;
use mysql::{Column, Row, QueryResult};
use std::result;
use serde::de;
use std::any::Any;
use serde_json::Value;

use rbatis_macro_derive::RbatisMacro;
use rbatis_macro::RbatisMacro;
use std::collections::HashMap;
use serde_json::Number;
use std::str::FromStr;
use serde::de::DeserializeOwned;
use std::ops::Deref;

impl Decoder for QueryResult<'_> {
    fn decode<T>(&mut self) -> Result<T, String> where T: DeserializeOwned + RbatisMacro {
        let mut js = serde_json::Value::Null;
        if T::decode_name() == "Vec" || T::decode_name() == "Array" || T::decode_name() == "Slice" || T::decode_name() == "LinkedList" {
            //is array json
            let mut vec_v = vec![];
            self.for_each(|item| {
                let act = decodeRow(&item.unwrap());
                vec_v.push(act);
            });
            js = serde_json::Value::Array(vec_v)
        } else {
            let mut result: Result<T, String> = Result::Err("[Rbatis] rows.affected_rows > 1,but decode one result!".to_string());
            //not array json
            let mut index = 0;
            self.for_each(|item| {
                if index > 1 {
                    return;
                }
                js = decodeRow(&item.unwrap());
                index = index + 1;
            });
            if index > 0 {
                return result;
            }
        }
        let decodeR = serde_json::from_value(js);
        if decodeR.is_ok() {
            return Result::Ok(decodeR.unwrap());
        } else {
            let e = decodeR.err().unwrap().to_string();
            return Result::Err(e);
        }
    }
}

pub fn decodeRow(row: &Row) -> Value {
    let cs = row.columns();
    let mut m = serde_json::map::Map::new();
    for c in cs.as_ref() {
        let columnName = c.name_str();
        let k = columnName.as_ref();
        let f: mysql::Value = row.get(k).unwrap();

        let mut sql = f.as_sql(true);
        let sqlLen = sql.len();
        let item: serde_json::Value;
        if sql.as_str() == "NULL" {
            item = serde_json::Value::Null;
        } else {
            if sql == "''" {
                sql = "\"\"".to_owned();
                item = serde_json::Value::String(sql);
            } else if sql.starts_with("'") {
                let slice = &sql[1..(sqlLen - 1)];
                sql = "\"".to_owned() + slice + "\"";
                item = serde_json::Value::String(sql);
            } else {
                let n = Number::from_str(sql.as_str());
                if n.is_ok() {
                    item = serde_json::Value::Number(n.unwrap());
                } else {
                    item = serde_json::Value::Null;
                }
            }
        }
        m.insert(columnName.to_string(), item);
    }
    return serde_json::Value::Object(m);
}