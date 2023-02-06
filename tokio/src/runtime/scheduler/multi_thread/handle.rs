use crate::future::Future;
use crate::loom::sync::Arc;
use crate::runtime::scheduler::multi_thread::worker;
use crate::runtime::{
    blocking, driver,
    task::{self, JoinHandle},
};
use crate::util::RngSeedGenerator;

use std::fmt;

/// Handle to the multi thread scheduler
pub(crate) struct Handle {
    /// Task spawner
    pub(super) shared: worker::Shared,

    /// Resource driver handles
    pub(crate) driver: driver::Handle,

    /// Blocking pool spawner
    pub(crate) blocking_spawner: blocking::Spawner,

    /// Current random number generator seed
    pub(crate) seed_generator: RngSeedGenerator,
}

impl Handle {
    /// Spawns a future onto the thread pool
    pub(crate) fn spawn<F>(me: &Arc<Self>, future: F, id: task::Id) -> JoinHandle<F::Output>
    where
        F: crate::future::Future + Send + 'static,
        F::Output: Send + 'static,
    {
        Self::bind_new_task(me, future, id)
    }

    pub(crate) fn shutdown(&self) {
        self.close();
    }

    pub(super) fn bind_new_task<T>(me: &Arc<Self>, future: T, id: task::Id) -> JoinHandle<T::Output>
    where
        T: Future + Send + 'static,
        T::Output: Send + 'static,
    {
        let (handle, notified) = me.shared.owned.bind(future, me.clone(), id);

        if let Some(notified) = notified {
            me.schedule_task(notified, false);
        }

        handle
    }
}

cfg_metrics! {
    use crate::runtime::{SchedulerMetrics, WorkerMetrics};

    impl Handle {
        pub(crate) fn num_workers(&self) -> usize {
            self.shared.worker_metrics.len()
        }

        pub(crate) fn num_blocking_threads(&self) -> usize {
            self.blocking_spawner.num_threads()
        }

        pub(crate) fn num_idle_blocking_threads(&self) -> usize {
            self.blocking_spawner.num_idle_threads()
        }

        pub(crate) fn scheduler_metrics(&self) -> &SchedulerMetrics {
            &self.shared.scheduler_metrics
        }

        pub(crate) fn worker_metrics(&self, worker: usize) -> &WorkerMetrics {
            &self.shared.worker_metrics[worker]
        }

        pub(crate) fn injection_queue_depth(&self) -> usize {
            self.shared.injection_queue_depth()
        }

        pub(crate) fn worker_local_queue_depth(&self, worker: usize) -> usize {
            self.shared.worker_local_queue_depth(worker)
        }

        pub(crate) fn blocking_queue_depth(&self) -> usize {
            self.blocking_spawner.queue_depth()
        }
    }
}

cfg_taskdump! {
    impl Handle {
        pub(in super::super) fn in_pause<F, R>(&self, f: F) -> Option<R> 
        where
            F: Send + FnOnce() -> R,
            R: Send,
        {
            use nix::{
                libc,
                sys::{
                    ptrace,
                    wait::{waitpid, WaitStatus},
                },
                unistd::{fork, read, write, ForkResult},
            };

            std::thread::scope(|s| s.spawn(|| {
                let shared = self.blocking_spawner.inner.shared.lock();
                let (on_pause_read_fd, on_pause_write_fd) = nix::unistd::pipe().unwrap();
                let (on_resume_read_fd, on_resume_write_fd) = nix::unistd::pipe().unwrap();

                match unsafe { fork() } {
                    Ok(ForkResult::Parent { child, .. }) => {
                        // wait for the ptracer to signal that all workers are stopped
                        assert_eq!(read(on_pause_read_fd, &mut [0]), Ok(1));

                        let result = f();

                        // signal to the ptracer that workers should be resumed
                        write(on_resume_write_fd, b"\n").ok();

                        assert_eq!(waitpid(child, None), Ok(WaitStatus::Exited(child, 0)));

                        Some(result)
                    }
                    Ok(ForkResult::Child) => {
                        let threads = &shared.worker_threads;
                        for thread in threads.values() {
                            ptrace::attach(thread.pid).unwrap();
                        }

                        // signal to the trace thread that all tasks have been stopped
                        write(on_pause_write_fd, b"\n").ok();

                        // wait for all tracing to be resumed
                        assert_eq!(read(on_resume_read_fd, &mut [0]), Ok(1));

                        for thread in threads.values() {
                            ptrace::detach(thread.pid, None).unwrap();
                        }

                        // don't attempt to unlock workers mutex from child
                        std::mem::forget(shared);

                        unsafe { libc::_exit(0) };
                    }
                    Err(_) => None,
                }
            }).join()).unwrap()
        }
    }
}

impl fmt::Debug for Handle {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("multi_thread::Handle { ... }").finish()
    }
}
