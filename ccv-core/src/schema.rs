use std::{fmt::Display, str::FromStr, sync::Arc};

use crate::error::{self, Result};
use async_recursion::async_recursion;
use futures::future::join_all;
use futures::lock::Mutex;
use reqwest::Url;
use serde_json::{Map, Value};

#[derive(Debug, Clone)]
pub enum ConfigKind {
    CloudConfig,
    NetworkConfig,
}

impl ConfigKind {
    fn url(&self) -> &str {
        match self {
            Self::CloudConfig => "https://raw.githubusercontent.com/canonical/cloud-init/main/cloudinit/config/schemas/versions.schema.cloud-config.json",
            Self::NetworkConfig => "https://raw.githubusercontent.com/canonical/cloud-init/main/cloudinit/config/schemas/schema-network-config-v1.json",
        }
    }
}

impl FromStr for ConfigKind {
    type Err = String;

    fn from_str(s: &str) -> std::prelude::v1::Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "cloudconfig" => Ok(Self::CloudConfig),
            "networkconfig" => Ok(Self::NetworkConfig),
            _ => Err(format!("Not a valid str variant: {}", s)),
        }
    }
}

impl Display for ConfigKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CloudConfig => write!(f, "cloudconfig"),
            Self::NetworkConfig => write!(f, "networkconfig"),
        }
    }
}

#[derive(Debug)]
pub struct Schema(serde_json::Value, ConfigKind);

impl Schema {
    pub async fn get(kind: ConfigKind) -> Result<Self> {
        let client = reqwest::Client::new();
        let resp = client.get(kind.url()).send().await?;
        let schema = resp.json::<serde_json::Value>().await?;

        let resolver = Arc::new(Mutex::new(Resolver::new()));
        Ok(Self(resolve(resolver, schema).await?, kind))
    }

    pub fn from_vendored() -> Result<Self> {
        static SCHEMA: &str =
            include_str!("../../schemas/versions.schema.cloud-config.resolved.1.json");
        let schema = serde_json::from_str(SCHEMA)?;
        Ok(Self(schema, ConfigKind::CloudConfig))
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

    async fn get(&self, url: &str) -> Result<Value> {
        // XXX: value type encoding both cases
        match url.parse::<Url>() {
            Ok(url) => Ok(self.client.get(url).send().await?.json().await?),
            Err(_) => {
                let pointer = if url.starts_with("#/$defs") {
                    let (_, pointer) = url.split_once("#/$defs").expect("Cannot panic");
                    pointer
                } else {
                    url
                };

                match self.get_def(pointer).await {
                    None => Err(crate::error::Error::LocalSchemaRefNotFound {
                        r#ref: pointer.to_owned(),
                    }),
                    Some(val) => Ok(val),
                }
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
async fn resolve(
    resolver: Arc<Mutex<Resolver>>,
    schema: serde_json::Value,
) -> Result<serde_json::Value> {
    // TODO: clean up
    match schema {
        serde_json::Value::Object(mut obj) => {
            let mut new_obj = Map::with_capacity(obj.len());

            // set definitions
            if let Some(defs) = obj.remove(DEFS) {
                resolver.lock().await.set_defs(defs).await;
            }

            for (mut key, mut val) in obj.into_iter() {
                if key == REF {
                    if let Value::String(val_str) = val.clone() {
                        if key.starts_with("#/$defs") {
                            let (_, pointer) = val_str.split_once("#/$defs").expect("Cannot panic");
                            let (_, new_key) = match pointer.rsplit_once('/') {
                                Some(x) => x,
                                None => {
                                    return Err(error::Error::LocalSchemaRefInvalidName {
                                        r#ref: val_str.to_owned(),
                                    });
                                }
                            };
                            key = new_key.to_string();
                            val = resolver.lock().await.get(&val_str).await?;
                        } else {
                            val = resolver.lock().await.get(&val_str).await?;
                            return resolve(resolver.clone(), val).await;
                        }
                    }
                    new_obj.insert(key, resolve(resolver.clone(), val).await?);
                } else {
                    new_obj.insert(key, resolve(resolver.clone(), val).await?);
                }
            }
            Ok(serde_json::Value::Object(new_obj))
        }
        serde_json::Value::Array(arr) => {
            let mut new_arr = Vec::with_capacity(arr.len());
            for item in join_all(arr.into_iter().map(|v| resolve(resolver.clone(), v))).await {
                new_arr.push(item?);
            }

            Ok(serde_json::Value::Array(new_arr))
        }
        _ => Ok(schema),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn fetch() {
        let schema = Schema::get(ConfigKind::CloudConfig)
            .await
            .expect("valid schema");
        println!("{}", serde_json::to_string_pretty(&schema.0).unwrap());
        // let mut file = std::fs::File::create("new_schema.json").unwrap();
        // write!(file, "{}", serde_json::to_string_pretty(&schema.0).unwrap()).unwrap();
    }
}
