use once_cell::sync::Lazy;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{OwnedSemaphorePermit, Semaphore};

/// Maximum number of concurrent SSH sessions allowed.
/// This can be tuned as needed.
const MAX_CONCURRENT: usize = 4;

pub struct SshLimiter {
    semaphore: Semaphore,
    host_locks: Mutex<HashMap<String, Arc<Mutex<()>>>>,
}

impl SshLimiter {
    const fn new(max: usize) -> Self {
        Self {
            semaphore: Semaphore::const_new(max),
            host_locks: Mutex::new(HashMap::new()),
        }
    }

    fn get_host_lock(&self, host_id: &str) -> Arc<Mutex<()>> {
        let mut map = self.host_locks.lock();
        map.entry(host_id.to_string())
            .or_insert_with(|| Arc::new(Mutex::new(())))
            .clone()
    }

    pub fn acquire(&self, host_id: &str) -> (OwnedSemaphorePermit, parking_lot::MutexGuard<'_, ()>) {
        let permit = tokio::runtime::Handle::current()
            .block_on(self.semaphore.acquire_owned())
            .expect("Semaphore closed");

        let host_lock = self.get_host_lock(host_id);
        let guard = host_lock.lock();
        (permit, guard)
    }
}

pub static SSH_LIMITER: Lazy<SshLimiter> = Lazy::new(|| SshLimiter::new(MAX_CONCURRENT));
