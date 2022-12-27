use std::{rc::Rc, sync::{Arc,}, borrow::BorrowMut};

use futures::future::join_all;
use async_recursion::async_recursion;
use reqwest::Url;
use serde_json::{Map, Value};
use futures::lock::Mutex;

use crate::validator::Validator;

#[derive(Debug)]
pub struct Schema(serde_json::Value);

impl Schema {
    pub async fn get() -> Self {
        let client = reqwest::Client::new();
        let resp = client.get("https://raw.githubusercontent.com/canonical/cloud-init/main/cloudinit/config/schemas/versions.schema.cloud-config.json").send().await.unwrap();
        let schema = resp.json::<serde_json::Value>().await.unwrap();

        let resolver = Arc::new(Mutex::new(Resolver::new()));
        Self(resolve(resolver, schema).await)
    }

    pub fn from_schema_store() -> Self {
        todo!();
    }
}

const REF: &str = "$ref";
const DEFS: &str = "$defs";

struct Resolver {
    client: reqwest::Client,
    defs: Arc<Mutex<Option<serde_json::Value>>>,
}

impl Resolver {
    fn new() -> Self {
        Resolver {
            client: reqwest::Client::new(),
            defs: Arc::new(Mutex::new(None)),
        }
    }

    async fn get(&self, url: &str) -> Value {
        dbg!(&url);
        match url.parse::<Url>() {
            Ok(url) => self.client.get(url).send().await.unwrap().json().await.unwrap(),
            Err(_) => {
                self.get_def(url).await.unwrap()
            }
        }
    }

    async fn set_defs(&self, defs: Value) {
        *self.defs.lock().await = Some(defs);
    }

    async fn get_def(&self, key: &str) -> Option<Value> {
        match self.defs.lock().await {
            Some(defs) => defs.pointer(key).cloned()
        }
    }

}

#[async_recursion]
async fn resolve(resolver: Arc<Mutex<Resolver>>, schema: serde_json::Value) -> serde_json::Value {
    match schema {
        serde_json::Value::Object(obj) => {
            let mut new_obj = Map::with_capacity(obj.len());
            if let Some(defs) = obj.remove(DEFS) {
                resolver.lock().await.set_defs(defs).await;
            }
            for (key, mut val) in obj.into_iter() {
                if key == DEFS {
                }
                else if key == REF {
                    val = resolver.lock().await.get(val.as_str().unwrap()).await;
                    val = resolve(resolver.clone(), val).await;
                }
                new_obj.insert(key, resolve(resolver.clone(), val).await);
            }
            serde_json::Value::Object(new_obj)
        }
        serde_json::Value::Array(arr) => serde_json::Value::Array(
            join_all(
                arr.into_iter()
                    .map(|v| resolve(resolver.clone(), v))
                    .collect::<Vec<_>>(),
            )
            .await,
        ),
        _ => schema,
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn fetch() {
        let schema = Schema::get().await;
        unimplemented!();
    }
}
