use crossbeam_channel::{unbounded, Receiver, Sender};
use std::{sync::Arc, thread};

// WASM-only exports — compiled only when targeting WebAssembly.
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
extern "C" {
    pub fn alert(s: &str);
}

/// Greet a user by name via a browser `alert`. Only available in WASM builds.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn greet(name: &str) {
    alert(&format!("Hello, {}!", name));
}

/// A fixed-size pool of worker threads that execute submitted jobs concurrently.
///
/// Jobs are submitted with [`ThreadPool::execute`] and dispatched to idle workers
/// via a `crossbeam-channel` queue.  When the pool is dropped, the channel is
/// closed so every worker finishes its current job and then exits cleanly.
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<Sender<Job>>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool {
    /// Create a new [`ThreadPool`] with `size` worker threads.
    ///
    /// # Panics
    ///
    /// Panics if `size` is zero.
    #[must_use]
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0, "ThreadPool size must be greater than zero");

        let (sender, receiver) = unbounded();

        // crossbeam's Receiver<T> is Sync, so all workers can share it via
        // Arc without a Mutex — no lock contention while waiting for jobs.
        let receiver = Arc::new(receiver);

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool {
            workers,
            sender: Some(sender),
        }
    }

    /// Returns the number of worker threads in this pool.
    #[must_use]
    pub fn size(&self) -> usize {
        self.workers.len()
    }

    /// Submit a job for execution on the next available worker.
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);

        if let Some(sender) = &self.sender {
            sender.send(job).expect("worker channel should be open");
        }
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        // Dropping the sender closes the channel; each worker's recv() call
        // will return Err once the queue is drained, signalling it to exit.
        self.sender.take();

        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Receiver<Job>>) -> Worker {
        let thread = thread::spawn(move || loop {
            // crossbeam Receiver::recv() is safe to call from multiple threads
            // simultaneously — no Mutex is required.
            match receiver.recv() {
                Ok(job) => {
                    println!("Worker {} got a job; executing.", id);
                    job();
                }
                Err(_) => {
                    println!("Worker {} disconnected; shutting down.", id);
                    break;
                }
            }
        });

        Worker {
            id,
            thread: Some(thread),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[test]
    #[should_panic(expected = "ThreadPool size must be greater than zero")]
    fn thread_pool_new_panics_on_zero() {
        let _ = ThreadPool::new(0);
    }

    #[test]
    fn thread_pool_size_matches_requested() {
        let pool = ThreadPool::new(4);
        assert_eq!(pool.size(), 4);
    }

    #[test]
    fn thread_pool_executes_all_jobs() {
        let pool = ThreadPool::new(2);
        let counter = Arc::new(Mutex::new(0u32));

        for _ in 0..10 {
            let counter = Arc::clone(&counter);
            pool.execute(move || {
                let mut n = counter.lock().unwrap();
                *n += 1;
            });
        }

        // Drop the pool to flush all pending jobs before asserting.
        drop(pool);

        assert_eq!(*counter.lock().unwrap(), 10);
    }

    #[test]
    fn thread_pool_shuts_down_cleanly() {
        // Should not deadlock or panic on drop.
        let pool = ThreadPool::new(4);
        drop(pool);
    }
}