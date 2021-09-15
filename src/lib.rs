use std::sync::mpsc::channel;
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::sync::Mutex;

pub struct ThreadPool {
    _handles: Vec<std::thread::JoinHandle<()>>,
    sender: Sender<Box<dyn FnMut() + Send>>,
}

impl ThreadPool {
    pub fn new(num_thread: u8) -> Self {
        let (sender, receiver) = channel::<Box<dyn FnMut() + Send>>();
        let receiver = Arc::new(Mutex::new(receiver));
        let mut _handles = vec![];

        for index in 0..num_thread {
            let receiver_clone = receiver.clone();
            let handle = std::thread::spawn(move || loop {
                let mut work = match receiver_clone.lock().unwrap().recv() {
                    Ok(work) => work,
                    Err(_) => break,
                };

                println!("Start thread {}", index);
                work();
                println!("Finish thread {}", index);
            });

            _handles.push(handle);
        }

        Self { _handles, sender }
    }

    pub fn execute<T: FnMut() + Send + 'static>(&self, work: T) {
        self.sender.send(Box::new(work)).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_works() {
        use std::sync::atomic::{AtomicI32, Ordering};

        let n = AtomicI32::new(0);
        let nref = Arc::new(n);
        let pool = ThreadPool::new(10);
        let nref_clone = nref.clone();
        let foo = move || {
            nref_clone.fetch_add(1, Ordering::SeqCst);
        };
        pool.execute(foo.clone());
        pool.execute(foo.clone());
        std::thread::sleep(std::time::Duration::from_secs(1));
        assert_eq!(nref.load(Ordering::SeqCst), 2);
    }
}
