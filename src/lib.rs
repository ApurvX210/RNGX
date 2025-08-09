use std::fmt::{Display, Formatter};
use std::mem::take;
use std::sync::{mpsc,Arc,Mutex};
use std::{thread, thread::JoinHandle};

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

type Job = Box<dyn FnOnce() + Send + 'static>;

struct Worker{
    id: usize,
    thread_handler: JoinHandle<()>
}

impl Worker {
    fn new(id:usize,reciever:Arc<Mutex<mpsc::Receiver<Job>>>,) -> Worker
    {
        print!("Heelo");
        let handler = thread::Builder::new().spawn(move ||
            {
                loop {
                    let message = reciever.lock().unwrap().recv();
                    match message {
                        Ok(job) => {
                            println!("Worker {id} got a job; executing.");
                            job();
                        },
                        Err(_) => {
                            println!("Worker {id} disconnected; shutting down.");
                            break;
                        }
                    }
                }
            }).expect("Not able to create a new thread");
        Worker { id: id, thread_handler:handler}
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());
        for worker in self.workers.drain(..){
            println!("Shutting down worker {}", worker.id);

            worker.thread_handler.join().expect("Error occured while gracefully shutting down the worker");
        }
    }
}

pub struct ThreadPool{
    workers: Vec<Worker>,
    sender:  Option<mpsc::Sender<Job>>
}

impl ThreadPool {
    /// Build a new ThreadPool.
    ///
    /// The size is the number of threads in the pool.
    ///
    /// # Panics
    ///
    /// The `new` function will panic if the size is zero.
    pub fn build(thread_count: usize) -> Result<ThreadPool,PoolCreationError> {
        if thread_count <= 0{
            return Err(PoolCreationError::ZeroThreadError);
        }
        
        let (sender,receiver) = mpsc::channel();

        let thread_reciever = Arc::new(Mutex::new(receiver));
        let mut workers: Vec<Worker> = Vec::with_capacity(thread_count);

        for id in 0..thread_count{
            let rec = Arc::clone(&thread_reciever);
            workers.push(Worker::new(id,rec));
        }

        Ok(ThreadPool { workers:workers, sender:Some(sender)})
    }

    pub fn execute<F>(&self,f: F)
        where
            F: FnOnce() + Send + 'static
    {
        let job = Box::new(f);

        self.sender.as_ref().unwrap().send(job).expect("Error Occured while executing the request");
    }

}