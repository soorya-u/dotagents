use crate::schema::cache::CacheConfig;

pub struct Settings {
    cache: CacheConfig,
}

impl Settings {
    pub fn new() -> Self {
        Self {
            cache: CacheConfig::default(),
        }
    }

    pub fn from_cache(cache: CacheConfig) -> Self {
        Self { cache }
    }

}
