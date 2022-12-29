// TODO: error handling
use std::sync::Arc;

use async_recursion::async_recursion;
use futures::future::join_all;
use futures::lock::Mutex;
use reqwest::Url;
use serde_json::{Map, Value};

#[derive(Debug)]
pub struct Schema(serde_json::Value);

impl Schema {
    pub async fn get() -> Self {
        let client = reqwest::Client::new();
        let resp = client.get(
            "https://raw.githubusercontent.com/canonical/cloud-init/main/cloudinit/config/schemas/versions.schema.cloud-config.json"
        ).send().await.unwrap();
        let schema = resp.json::<serde_json::Value>().await.unwrap();

        let resolver = Arc::new(Mutex::new(Resolver::new()));
        Self(resolve(resolver, schema).await)
    }

    #[allow(dead_code)]
    pub fn from_vendored() -> Self {
        static SCHEMA: &str =
            include_str!("../schemas/versions.schema.cloud-config.resolved.1.json");
        let schema = serde_json::from_str(SCHEMA).expect("valid json");
        Self(schema)
    }

    pub fn schema(&self) -> &Value {
        &self.0
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

    async fn get(&self, url: &str) -> Option<Value> {
        // dbg!(&url);
        match url.parse::<Url>() {
            Ok(url) => self
                .client
                .get(url)
                .send()
                .await
                .unwrap()
                .json()
                .await
                .unwrap(),
            Err(_) => {
                let pointer = if url.starts_with("#/$defs") {
                    let (_, pointer) = url.split_once("#/$defs").expect("Cannot panic");
                    pointer
                } else {
                    url
                };

                self.get_def(pointer).await
            }
        }
    }

    async fn set_defs(&self, defs: Value) {
        *self.defs.lock().await = Some(defs);
    }

    async fn get_def(&self, key: &str) -> Option<Value> {
        match &*self.defs.lock().await {
            Some(defs) => defs.pointer(key).cloned(),
            None => None,
        }
    }
}

#[async_recursion]
async fn resolve(resolver: Arc<Mutex<Resolver>>, schema: serde_json::Value) -> serde_json::Value {
    // TODO: clean up
    match schema {
        serde_json::Value::Object(mut obj) => {
            let mut new_obj = Map::with_capacity(obj.len());

            // set definitions
            if let Some(defs) = obj.remove(DEFS) {
                resolver.lock().await.set_defs(defs).await;
            }

            for (mut key, mut val) in obj.into_iter() {
                //dbg!(&key);
                //dbg!(&val);
                if key == REF {
                    if let Value::String(val_str) = val.clone() {
                        if key.starts_with("#/$defs") {
                            let (_, pointer) = val_str.split_once("#/$defs").expect("Cannot panic");
                            let (_, new_key) = pointer
                                .rsplit_once('/')
                                .expect("capture key name in pointer");
                            key = new_key.to_string();
                        } else {
                            val = resolver
                                .lock()
                                .await
                                .get(val.as_str().unwrap())
                                .await
                                .expect("value not found");
                            return resolve(resolver.clone(), val).await;
                        }
                    }
                    val = resolver
                        .lock()
                        .await
                        .get(val.as_str().unwrap())
                        .await
                        .expect("value not found");
                    new_obj.insert(key, resolve(resolver.clone(), val).await);
                    // dbg!(&new_obj);
                } else {
                    new_obj.insert(key, resolve(resolver.clone(), val).await);
                    // dbg!(&new_obj);
                }
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
        println!("{}", serde_json::to_string_pretty(&schema.0).unwrap());
        // let mut file = std::fs::File::create("new_schema.json").unwrap();
        // write!(file, "{}", serde_json::to_string_pretty(&schema.0).unwrap()).unwrap();
    }
}
