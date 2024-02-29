use std::sync::{Arc, mpsc, Mutex};
use std::thread;

pub struct ThreadPool {
    sender: mpsc::Sender<Job>,
}

impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        // let mut workers = Vec::with_capacity(size);
        for id in 0..size {
            Worker::new(id, Arc::clone(&receiver));
        }
        ThreadPool { sender }
    }
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender.send(job).unwrap();
    }

}

struct Worker {
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        /* let thread = */ thread::spawn(move || loop {
            let job = receiver.lock().unwrap().recv().unwrap();
            println!("Worker {} got a job; executing.", id);
            job();
            println!("Worker {} finished the job.", id)
        });
        Worker { }
    }
}

type Job = Box<dyn FnOnce() + Send + 'static>;