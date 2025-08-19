#[cfg(feature = "db")]
use diesel::{AsChangeset, Insertable, Queryable};

use serde::{Deserialize, Serialize};

#[cfg(feature = "db")]
use crate::cache_schema::cache;

#[derive(Deserialize, Serialize, Default, Clone, Debug)]
#[cfg_attr(feature = "db", derive(Insertable, Queryable, AsChangeset))]
#[cfg_attr(feature = "db", diesel(table_name = cache))]

pub struct CacheModel {
    pub id: Option<i32>,
    pub url: String,
    pub blob: Vec<u8>,
    pub expires: i64,
}
