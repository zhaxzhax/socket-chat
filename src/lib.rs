use std::io::prelude::*;
use std::net::TcpStream;
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
#[derive(Debug)]
pub struct StreamPool {
    stream_list: Arc<Vec<Mutex<Option<TcpStream>>>>,
    allow_num: usize,
    stream_num: usize,
}
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Message>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

enum Message {
    NewJob(Job),
    Terminate,
}

impl StreamPool {
    pub fn new(size: usize) -> StreamPool {
        let mut stream_list = Vec::new();
        let allow_num = size;
        assert!(size > 0);
        for _ in 0..size {
            stream_list.push(Mutex::new(None))
        }
        let stream_list = Arc::new(stream_list);
        let stream_num = 0;
        StreamPool {
            stream_list,
            allow_num,
            stream_num,
        }
    }
    pub fn connect(&mut self, stream: TcpStream) {
        assert!(self.stream_num < self.allow_num);
        let mut mutex = self.stream_list[self.stream_num].lock().unwrap();
        *mutex = Some(stream);
        self.stream_num = self.stream_num + 1;
    }
    pub fn begin(&mut self) {
        let mut handles = vec![];
        for id in 0..self.allow_num {
            let stream_list = Arc::clone(&self.stream_list);
            let thread = thread::spawn(move || {
                let mut buffer = [0; 512];
                loop {
                    let mut mutex = stream_list[id].lock().unwrap();
                    match &mut *mutex {
                        Some(stream) => {
                            let debug = stream.read(&mut buffer).unwrap();
                            println!("没有被阻塞");
                            println!("{:?}", debug);
                            let hold = b"holding";
                            move || {
                                let mutex_release = mutex;
                                println!("销毁锁")
                            };
                            match buffer.starts_with(hold) {
                                true => {
                                    println!("hold!");
                                }
                                _ => {
                                    let mut for_index: usize = 0;
                                    println!("{:?}", stream_list.iter());
                                    for item in stream_list.iter() {
                                        println!("在获取锁");
                                        let mut write_mutex = item.lock().unwrap();
                                        if for_index == id {
                                            continue;
                                        }
                                        match &mut *write_mutex {
                                            Some(stream) => {
                                                stream.write(&mut buffer).unwrap();
                                                println!("写了些东西");
                                                stream.flush().unwrap();
                                            }
                                            None => {}
                                        }
                                        for_index = for_index + 1;
                                    }
                                }
                            }
                        }
                        None => {}
                    }
                }
            });
            handles.push(thread);
        }
        for handle in handles {
            handle.join().unwrap();
        }
    }
}
impl ThreadPool {
    /// Create a new ThreadPool.
    ///
    /// The size is the number of threads in the pool.
    ///
    /// # Panics
    ///
    /// The `new` function will panic if the size is zero.
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();

        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }
        ThreadPool { workers, sender }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);

        self.sender.send(Message::NewJob(job)).unwrap();
    }
    pub fn listen(&mut self) {}
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        println!("Sending terminate message to all workers.");

        for _ in &self.workers {
            self.sender.send(Message::Terminate).unwrap();
        }

        println!("Shutting down all workers.");

        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

pub struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv().unwrap();
            match message {
                Message::NewJob(job) => {
                    println!("Worker {} got a job; executing.", id);
                    job();
                }
                Message::Terminate => {
                    println!("Worker {} was told to terminate.", id);
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
