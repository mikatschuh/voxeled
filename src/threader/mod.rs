/* threading goals

verschiedene Arten von Tasks:

Priority Tasks:

    Werden versucht sofort auszuführen. Ignoriert alle anderen Tasks.

Casual Tasks:

    Werden normal ausgeführt.

No Priority Tasks:

    Werden ausgeführt sofern Zeit zu Verfügung steht.
*/
use crossbeam::channel::{bounded, Sender};
use crossbeam::deque::Injector;

use std::{sync::Arc, thread};
pub mod task;
use task::Task;

pub struct Threadpool {
    workers: Vec<thread::JoinHandle<()>>,
    pub priority_queue: Arc<Injector<Task>>,
    normal_queues: [Arc<Injector<Task>>; 2],
    waker: Sender<bool>,
}
impl Threadpool {
    pub fn new(num_threads: Option<usize>) -> Self {
        let num_threads = num_threads.unwrap_or_else(|| num_cpus::get());

        // Create queues
        let priority_queue = Arc::new(Injector::<Task>::new());
        let normal_queues = [
            Arc::new(Injector::<Task>::new()),
            Arc::new(Injector::<Task>::new()),
        ];

        let mut workers = Vec::with_capacity(num_threads);
        let (waker, wake_call) = bounded::<bool>(1);

        for i in 0..num_threads {
            let priority_queue = priority_queue.clone();
            let normal_queues = normal_queues.clone();

            let wake_call = wake_call.clone();

            workers.push(thread::spawn(move || {
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
                        match wake_call.recv() {
                            Ok(false) => {
                                println!("#{}: terminated", i);
                                break;
                            }
                            Err(_) => break,
                            _ => println!("#{}: woke up!", i),
                        }
                    }
                }
            }));
        }

        Threadpool {
            workers,
            priority_queue,
            normal_queues,
            waker,
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
