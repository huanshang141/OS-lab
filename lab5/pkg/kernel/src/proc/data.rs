use super::*;
use crate::proc::sync::SemaphoreSet;
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
    pub(super) semaphore: Arc<RwLock<SemaphoreSet>>,
}

impl Default for ProcessData {
    fn default() -> Self {
        Self {
            env: Arc::new(RwLock::new(BTreeMap::new())),
            resource: Arc::new(RwLock::new(ResourceSet::default())),
            semaphore: Arc::new(RwLock::new(SemaphoreSet::default())),
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
    pub fn new_sem(&self, key: u32, val: usize) -> bool {
        self.semaphore.write().insert(key, val)
    }
    pub fn remove_sem(&self, key: u32) -> bool {
        self.semaphore.write().remove(key)
    }
    pub fn sem_signal(&self, key: u32) -> SemaphoreResult {
        self.semaphore.write().signal(key)
    }
    pub fn sem_wait(&self, key: u32, pid: ProcessId) -> SemaphoreResult {
        self.semaphore.write().wait(key, pid)
    }
}
