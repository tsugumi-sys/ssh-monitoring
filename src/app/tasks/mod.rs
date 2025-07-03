pub mod executor;
pub mod task;

pub mod cpu_status_task;
pub mod disk_task;
pub mod gpu_task;
pub mod os_task;
pub mod ssh_status_task;

use once_cell::sync::Lazy;
use tokio::sync::Semaphore;

/// Limits the number of concurrent SSH connections.
pub static SSH_SEMAPHORE: Lazy<Semaphore> = Lazy::new(|| Semaphore::new(8));
