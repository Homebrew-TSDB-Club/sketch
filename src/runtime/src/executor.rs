use std::cell::{OnceCell, RefCell};
use std::collections::VecDeque;
use std::future::Future;
use std::rc::Rc;
use std::sync::Arc;

use async_task::{Runnable, Task};
use concurrent_queue::ConcurrentQueue;
use futures_lite::future;

const NR_TASKS: usize = 256;

thread_local! {
    pub(crate) static CONTEXT: OnceCell<Context> = OnceCell::new()
}

#[derive(Debug)]
pub(crate) struct Context {
    pub(crate) local: Rc<RefCell<VecDeque<Runnable>>>,
    pub(crate) assigned: Arc<ConcurrentQueue<Runnable>>,
    pub(crate) global: Arc<ConcurrentQueue<Runnable>>,
}

impl Context {
    fn new(global: Arc<ConcurrentQueue<Runnable>>, assigned: Arc<ConcurrentQueue<Runnable>>) -> Self {
        Context {
            local: Rc::new(RefCell::new(VecDeque::new())),
            assigned,
            global,
        }
    }
}

#[derive(Debug)]
pub struct Executor;

impl Executor {
    /// Creates a new executor.
    pub fn new(global: Arc<ConcurrentQueue<Runnable>>, assigned: Arc<ConcurrentQueue<Runnable>>) -> Self {
        CONTEXT.with(|context| context.set(Context::new(global, assigned)).unwrap());
        Executor {}
    }

    pub async fn run(&mut self, future: impl Future<Output = ()>) {
        // A future that runs tasks forever.
        let run_forever = async move {
            loop {
                CONTEXT.with(|context| {
                    let context = context.get().unwrap();
                    let mut capacity = NR_TASKS;
                    for _ in 0..(capacity / 2) {
                        let runnable = context.local.borrow_mut().pop_front();
                        if let Some(runnable) = runnable {
                            runnable.run();
                            capacity -= 1;
                        } else {
                            break;
                        }
                    }
                    for _ in 0..(capacity / 2) {
                        if let Ok(runnable) = context.assigned.pop() {
                            runnable.run();
                            capacity -= 1;
                        } else {
                            break;
                        }
                    }
                    for _ in 0..capacity {
                        if let Ok(runnable) = context.global.pop() {
                            runnable.run();
                        } else {
                            break;
                        }
                    }
                });

                future::yield_now().await;
            }
        };

        // Run `future` and `run_forever` concurrently until `future` completes.
        future::or(future, run_forever).await;
    }
}

pub fn spawn_local<T>(future: impl Future<Output = T>) -> Task<T> {
    let schedule = CONTEXT.with(|context| {
        let context = context.get().unwrap();
        let queue = Rc::clone(&context.local);
        move |runnable| {
            queue.borrow_mut().push_back(runnable);
        }
    });
    let (runnable, task) = unsafe { async_task::spawn_unchecked(future, schedule) };
    runnable.schedule();
    task
}

pub fn spawn<T: Send + Sync>(future: impl Future<Output = T> + Send + Sync) -> Task<T> {
    let schedule = CONTEXT.with(|context| {
        let context = context.get().unwrap();
        let global = Arc::clone(&context.global);
        move |runnable| {
            global.push(runnable).unwrap();
        }
    });
    let (runnable, task) = unsafe { async_task::spawn_unchecked(future, schedule) };
    runnable.schedule();
    task
}

#[cfg(test)]
mod test {
    use super::{spawn_local, Executor};
    use concurrent_queue::ConcurrentQueue;
    use futures_lite::future;
    use futures_lite::future::yield_now;
    use std::cell::RefCell;
    use std::rc::Rc;
    use std::sync::Arc;

    #[test]
    fn test_runtime() {
        let mut ex = Executor::new(
            Arc::new(ConcurrentQueue::unbounded()),
            Arc::new(ConcurrentQueue::unbounded()),
        );

        let task = spawn_local(async { 1 + 2 });
        future::block_on(ex.run(async {
            let res = task.await * 2;
            assert_eq!(res, 6);
        }));
    }

    #[test]
    fn test_yield() {
        let mut ex = Executor::new(
            Arc::new(ConcurrentQueue::unbounded()),
            Arc::new(ConcurrentQueue::unbounded()),
        );

        let counter = Rc::new(RefCell::new(0));
        let counter1 = Rc::clone(&counter);
        let task = spawn_local(async {
            {
                let mut c = counter1.borrow_mut();
                assert_eq!(*c, 0);
                *c = 1;
            }
            let counter_clone = Rc::clone(&counter1);
            let t = spawn_local(async {
                {
                    let mut c = counter_clone.borrow_mut();
                    assert_eq!(*c, 1);
                    *c = 2;
                }
                yield_now().await;
                {
                    let mut c = counter_clone.borrow_mut();
                    assert_eq!(*c, 3);
                    *c = 4;
                }
            });
            yield_now().await;
            {
                let mut c = counter1.borrow_mut();
                assert_eq!(*c, 2);
                *c = 3;
            }
            t.await;
        });
        future::block_on(ex.run(task));
        assert_eq!(*counter.as_ref().borrow(), 4);
    }
}
