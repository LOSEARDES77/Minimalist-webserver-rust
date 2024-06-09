use std::sync::{mpsc, Arc, Mutex};
use std::thread::{self, JoinHandle};

pub struct ThreadPool {
    sender: Option<mpsc::Sender<Message>>,
    workers: Option<Vec<Worker>>,
}

enum Message {
    NewJob(Job),
    Terminate,
}

impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers = Vec::with_capacity(size);
        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }
        ThreadPool {
            sender: Some(sender),
            workers: Some(workers),
        }
    }
    pub fn execute<F>(&self, f: F) -> Result<(), String>
    where
        F: FnOnce() + Send + 'static,
    {
        if self.sender.is_none() {
            return Err("ThreadPool is not initialized No Sender".to_string());
        }

        if self.workers.is_none() {
            return Err("ThreadPool is not initialized (No Workers)".to_string());
        }

        let job = Box::new(f);

        let res = match self.sender.as_ref().unwrap().send(Message::NewJob(job)) {
            Ok(_) => Ok(()),
            Err(e) => Err("Error sending job to worker \"".to_string() + &e.to_string() + "\"\n"),
        };

        res
    }

    pub fn shutdown(&mut self) -> Result<(), String> {
        if self.workers.is_none() {
            return Err("ThreadPool is not initialized (No Workers)".to_string());
        }

        for _ in 0..self.workers.as_ref().unwrap().len() {
            self.sender
                .as_ref()
                .unwrap()
                .send(Message::Terminate)
                .unwrap();
        }

        for worker in self.workers.as_mut().unwrap() {
            worker.join();
        }

        self.workers = None;
        self.sender = None;

        Ok(())
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        if self.workers.is_some() {
            for _ in 0..self.workers.as_ref().unwrap().len() {
                self.sender
                    .as_ref()
                    .unwrap()
                    .send(Message::Terminate)
                    .unwrap();
            }

            for worker in self.workers.as_mut().unwrap() {
                worker.join();
            }

            self.workers = None;
            self.sender = None;
        }
    }
}

struct Worker {
    thread: Option<JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let job = receiver.lock().unwrap().recv().unwrap();
            match job {
                Message::NewJob(job) => {
                    println!("Worker {} got a job; executing.", id);
                    job();
                }
                Message::Terminate => {
                    println!("Worker {} was told to terminate.", id);
                    break;
                }
            }
            println!("Worker {} finished the job.", id)
        });

        Worker {
            thread: Some(thread),
        }
    }

    fn join(&mut self) {
        if let Some(thread) = self.thread.take() {
            thread.join().unwrap();
        }
    }
}

type Job = Box<dyn FnOnce() + Send + 'static>;
