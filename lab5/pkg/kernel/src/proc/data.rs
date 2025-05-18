use super::*;
use crate::utils::resource::ResourceSet;
use alloc::{collections::BTreeMap, sync::Arc};
use spin::RwLock;
use x86_64::structures::paging::{
    Page,
    page::{PageRange, PageRangeInclusive},
};

#[derive(Debug, Clone)]
pub struct ProcessData {
    // shared data
    pub(super) env: Arc<RwLock<BTreeMap<String, String>>>,
    pub(super) resource: Arc<RwLock<ResourceSet>>,
}

impl Default for ProcessData {
    fn default() -> Self {
        Self {
            env: Arc::new(RwLock::new(BTreeMap::new())),
            resource: Arc::new(RwLock::new(ResourceSet::default())),
        }
    }
}

impl ProcessData {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn env(&self, key: &str) -> Option<String> {
        self.env.read().get(key).cloned()
    }

    pub fn set_env(&mut self, key: &str, val: &str) {
        self.env.write().insert(key.into(), val.into());
    }
    pub fn write(&self, fd: u8, buf: &[u8]) -> isize {
        self.resource.read().write(fd, buf)
    }
    pub fn read(&self, fd: u8, buf: &mut [u8]) -> isize {
        self.resource.read().read(fd, buf)
    }
}
