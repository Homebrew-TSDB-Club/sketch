#![warn(missing_debug_implementations)]
#![feature(once_cell)]
#![feature(can_vector)]

pub mod error;
mod executor;

use std::future::Future;
use std::sync::Arc;
use std::thread::{self, JoinHandle};

use async_task::{Runnable, Task};
use concurrent_queue::ConcurrentQueue;
pub use core_affinity::{get_core_ids, CoreId};
use error::RuntimeError;
pub use executor::spawn_local;
use executor::Executor;
use futures::channel::oneshot;
use futures_lite::future;

#[derive(Debug)]
pub struct Runtime {
    cores: Vec<CoreId>,
    executors: Vec<ExecutorHandler>,
    global_tasks: Arc<ConcurrentQueue<Runnable>>,
}

unsafe impl Send for Runtime {}
unsafe impl Sync for Runtime {}

impl Runtime {
    pub fn new(require_cores: &[CoreId]) -> Result<Self, RuntimeError> {
        let mut inuse = Vec::new();
        let all_cores = get_core_ids().ok_or(RuntimeError::GetCoreError)?;
        for require in require_cores {
            let mut has = false;
            for core in &all_cores {
                if core.id == require.id {
                    has = true;
                    break;
                }
            }
            if !has {
                return Err(RuntimeError::NotMuchCores {
                    require: require.id,
                    has: inuse.len(),
                });
            }
            inuse.push(*require);
        }
        Ok(Self {
            cores: inuse,
            executors: Vec::new(),
            global_tasks: Arc::new(ConcurrentQueue::unbounded()),
        })
    }

    pub fn run(&mut self) {
        for &id in &self.cores {
            let assigned = Arc::new(ConcurrentQueue::unbounded());
            let local_assigned = Arc::clone(&assigned);
            let local_global = Arc::clone(&self.global_tasks);
            let (closer, recv) = oneshot::channel::<()>();
            let join = thread::spawn(move || {
                core_affinity::set_for_current(id);
                let mut ex = Executor::new(local_global, local_assigned);
                future::block_on(ex.run(async move {
                    recv.await.unwrap();
                }));
            });

            self.executors.push(ExecutorHandler { join, assigned, closer });
        }
    }

    pub fn spawn<T: Send + Sync + 'static>(&self, future: impl Future<Output = T> + Send + Sync + 'static) -> Task<T> {
        let global_tasks = Arc::clone(&self.global_tasks);
        let schedule = move |runnable| {
            global_tasks.push(runnable).unwrap();
        };
        let (runnable, task) = unsafe { async_task::spawn_unchecked(future, schedule) };
        runnable.schedule();
        task
    }

    pub fn spawn_to<T: Send + 'static, F: Future<Output = T> + 'static>(
        &self,
        id: usize,
        future: impl (FnOnce() -> F) + Send,
    ) -> Task<T> {
        let assigned = Arc::clone(&self.executors[id].assigned);
        let schedule = move |runnable| {
            assigned.push(runnable).unwrap();
        };
        let (runnable, task) = unsafe { async_task::spawn_unchecked(future(), schedule) };
        runnable.schedule();
        task
    }
}

impl Drop for Runtime {
    fn drop(&mut self) {
        for ex in self.executors.drain(..) {
            ex.closer.send(()).unwrap();
            ex.join.join().unwrap();
        }
    }
}

#[derive(Debug)]
struct ExecutorHandler {
    join: JoinHandle<()>,
    assigned: Arc<ConcurrentQueue<Runnable>>,
    closer: oneshot::Sender<()>,
}

#[cfg(test)]
mod tests {
    use super::spawn_local;
    use crate::Runtime;
    use core_affinity::CoreId;
    use futures_lite::future;
    use std::cell::Cell;
    use std::rc::Rc;

    #[test]
    fn test_runtime() {
        let cores = (0..1).map(|id| CoreId { id }).collect::<Vec<_>>();
        let mut runtime = Runtime::new(&cores).unwrap();
        runtime.run();
        future::block_on(runtime.spawn_to(0, || async {
            let printable = Rc::new(Cell::new(1));
            let p_clone = Rc::clone(&printable);
            spawn_local(async { p_clone.set(p_clone.get() + 1) }).await;
            println!("{}", printable.get());
        }));
    }
}
