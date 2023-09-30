use anyhow::{anyhow, Result};
use r2d2::Pool;
use r2d2_memcache::MemcacheConnectionManager;

/// Represents the value to be stored in Memcached, which can be either a string or a number.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum SetValue {
    /// Stores a value as a string of characters.
    Characters(String),
    /// Stores a value as a 16-bit unsigned number.
    Number(u16),
}

/// Define a structure to manage the Memcached connection pool.
#[derive(Clone, Debug)]
pub struct MemPool {
    /// Optional pool of Memcached connections.
    pub connection: Option<Pool<MemcacheConnectionManager>>,
}

/// Define a trait for the MemcacheManager with methods to interact with Memcached.
pub trait MemcacheManager {
    /// Get data from a given key.
    fn get<T: ToString>(&self, key: T) -> Result<Option<String>>;
    /// Set data in Memcached and return the key.
    fn set<T: ToString>(&self, key: T, value: SetValue) -> Result<String>;
    /// Delete data based on the key.
    fn delete<T: ToString>(&self, key: T) -> Result<()>;
}

impl MemcacheManager for MemPool {
    /// Retrieve data from Memcached based on the key.
    fn get<T: ToString>(&self, key: T) -> Result<Option<String>> {
        let connection = self
            .connection
            .as_ref()
            .ok_or_else(|| anyhow!("No connection pool"))?
            .get()
            .map_err(|error| {
                log::error!("Error while getting connection: {:?}", error);
                error
            })?;

        connection
            .get(&key.to_string())
            .map(|data| {
                log::trace!("Cache data got with key {}", key.to_string());
                data
            })
            .map_err(|error| {
                log::error!("Error while retrieving data: {:?}", error);
                error.into()
            })
    }

    /// Store data in Memcached and return the key.
    fn set<T: ToString>(&self, key: T, value: SetValue) -> Result<String> {
        let connection = self
            .connection
            .as_ref()
            .ok_or_else(|| anyhow!("No connection pool"))?
            .get()
            .map_err(|error| {
                log::error!("Error while getting connection: {:?}", error);
                error
            })?;

        let result = match value.clone() {
            SetValue::Characters(data) => connection.set(&key.to_string(), data, 300),
            SetValue::Number(data) => connection.set(&key.to_string(), data, 300),
        };

        result
            .map(move |_| {
                log::trace!(
                    "Cache data set with key {} and content as {:?}",
                    key.to_string(),
                    value
                );
                key.to_string()
            })
            .map_err(|error| {
                log::error!("Error while setting data: {:?}", error);
                error.into()
            })
    }

    /// Delete data from Memcached based on the key.
    fn delete<T: ToString>(&self, key: T) -> Result<()> {
        let connection = self
            .connection
            .as_ref()
            .ok_or_else(|| anyhow!("No connection pool"))?
            .get()
            .map_err(|error| {
                log::error!("Error while getting connection: {:?}", error);
                error
            })?;

        connection
            .delete(&key.to_string())
            .map(move |_| {
                log::trace!("Cache deleted with key {}", key.to_string());
            })
            .map_err(|error| {
                log::error!("Error while deleting data: {:?}", error);
                error
            })?;

        Ok(())
    }
}

/// Initialize the connection pool for Memcached.
pub fn init(config: &crate::model::config::Config) -> Result<Pool<MemcacheConnectionManager>> {
    let manager = r2d2_memcache::MemcacheConnectionManager::new(format!(
        "memcache://{}?timeout=2&use_udp=true",
        config.database.memcached.hosts[0]
    ));

    Ok(r2d2_memcache::r2d2::Pool::builder()
        .max_size(config.database.memcached.pool_size)
        .min_idle(Some(2))
        .build(manager)?)
}
