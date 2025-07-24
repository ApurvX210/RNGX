use std::fmt::{Display, Error, Formatter};
use std::thread::spawn;
use std::{thread, thread::JoinHandle};
use std::any::Any;

#[derive(Debug)]
pub enum PoolCreationError {
    ZeroThreadError,
    TooManyThreads(usize),
}

impl Display for PoolCreationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PoolCreationError::ZeroThreadError => write!(f,"No. of thread cannot be zero"),
            PoolCreationError::TooManyThreads(thread_count) => write!(f, "Too many threads requested: {}", thread_count)
        }
    }
}
impl std::error::Error for PoolCreationError {}

struct Worker{
    id: usize,
    thread_handler: JoinHandle<()>
}

impl Worker {
    fn new(id:usize) -> Worker
    {
        let handler = thread::Builder::new().spawn(||{}).expect("Not able to create a new thread");
        Worker { id: id, thread_handler:handler }
    }
}

pub struct ThreadPool{
    workers: Vec<Worker>
}

impl ThreadPool {
    /// Build a new ThreadPool.
    ///
    /// The size is the number of threads in the pool.
    ///
    /// # Panics
    ///
    /// The `new` function will panic if the size is zero.
    fn build(thread_count: usize) -> Result<ThreadPool,PoolCreationError> {
        if thread_count <= 0{
            return Err(PoolCreationError::ZeroThreadError);
        }

        let mut workers: Vec<Worker> = Vec::with_capacity(thread_count);

        for id in 0..thread_count{
            workers.push(Worker::new(id));
        }

        Ok(ThreadPool { workers:workers})
    }

    fn execute<F>(&mut self,f: F)
        where
            F: FnOnce() + Send + 'static
    {
        if self.available_threads > 0{
            self.available_threads -= 1;
            thread::spawn(f);
        }else {
            
        }

    }

}