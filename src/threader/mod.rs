use crossbeam::atomic::AtomicCell;
/* threading goals

verschiedene Arten von Tasks:

Priority Tasks:

    Werden versucht sofort auszuführen. Ignoriert alle anderen Tasks.

Casual Tasks:

    Werden normal ausgeführt.

No Priority Tasks:

    Werden ausgeführt sofern Zeit zu Verfügung steht.
*/
use crossbeam::channel::{bounded, Receiver, Sender};
use crossbeam::deque::Injector;

use std::time::Instant;
use std::{sync::Arc, thread};
pub mod task;
use task::Task;

pub struct Threadpool {
    workers: Vec<thread::JoinHandle<()>>,
    sleeping: Arc<AtomicCell<usize>>,
    last_update: Instant,

    pub priority_queue: Arc<Injector<Task>>,
    normal_queues: [Arc<Injector<Task>>; 2],
    waker: Sender<bool>,
    wake_call: Receiver<bool>,
}
impl Threadpool {
    pub fn new() -> Self {
        let (waker, wake_call) = bounded(1);

        Threadpool {
            workers: Vec::new(),
            sleeping: Arc::new(AtomicCell::new(0)),
            last_update: Instant::now(),

            priority_queue: Arc::new(Injector::<Task>::new()),
            normal_queues: [
                Arc::new(Injector::<Task>::new()),
                Arc::new(Injector::<Task>::new()),
            ],
            waker,
            wake_call,
        }
    }
    pub fn launch(&mut self, num_threads: Option<usize>) {
        let num_threads = num_threads.unwrap_or_else(|| num_cpus::get() - 1);

        for i in 0..num_threads {
            let priority_queue = self.priority_queue.clone();
            let normal_queues = self.normal_queues.clone();

            let wake_call = self.wake_call.clone();
            let sleeping = self.sleeping.clone();

            self.workers.push(thread::spawn(move || {
                let mut poll = 0_usize;
                loop {
                    // Always handle ALL priority tasks first
                    while let Some(task) = priority_queue.steal().success() {
                        task.execute(i);
                    }

                    // Only handle normal/second queue tasks when no priority tasks exist
                    let mut counter = 0;
                    while let Some(task) = if counter < 3 {
                        normal_queues[0].steal().success().or_else(|| {
                            counter = 0;
                            normal_queues[1].steal().success()
                        })
                    } else if let Some(task) = normal_queues[1].steal().success() {
                        counter = 0;
                        Some(task)
                    } else {
                        normal_queues[0].steal().success()
                    } {
                        task.execute(i);
                        counter += 1;
                    }
                    if poll < 2 {
                        poll += 1;
                    } else {
                        poll = 0;
                        println!("#{}: no tasks anymore", i);
                        sleeping.fetch_add(1);
                        match wake_call.recv() {
                            Ok(false) | Err(_) => break,
                            _ => println!("#{}: woke up!", i),
                        }
                        sleeping.fetch_sub(1);
                    }
                }
                sleeping.fetch_sub(1);
                println!("#{}: terminated", i)
            }));
        }
    }
    pub fn update(&mut self) {
        if self.last_update.elapsed().as_nanos() >= 500_000_000 {
            let available_threads = num_cpus::get() as i64 - 1 - self.sleeping.load() as i64;
            println!(
                "threadpool outdated, checking for how many threads are available right now: {}",
                available_threads
            );
            if available_threads > 0 {
                self.launch(Some(available_threads as usize));
            } else if available_threads < 0 {
                for _ in 0..self.workers.len().max(-available_threads as usize) {
                    let _ = self.waker.send(false);
                }
            }
            self.last_update = Instant::now()
        }
    }
    pub fn add_to_first(&mut self, task: Task) {
        self.normal_queues[0].push(task);
        let _ = self.waker.send(true);
    }
    pub fn add_to_second(&mut self, task: Task) {
        self.normal_queues[1].push(task);
        let _ = self.waker.send(true);
    }
    pub fn add_priority(&mut self, task: Task) {
        self.priority_queue.push(task);
        let _ = self.waker.send(true);
    }
    pub fn drop(&mut self) {
        for _ in 0..self.workers.len() {
            let _ = self.waker.send(false);
        }
        while let Some(worker) = self.workers.pop() {
            if let Err(e) = worker.join() {
                println!("thread {} panicked, message: {:?}", self.workers.len(), e)
            };
        }
    }
}
