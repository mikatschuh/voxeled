/* threading goals

verschiedene Arten von Tasks:

Priority Tasks:

    Werden versucht sofort auszuführen. Ignoriert alle anderen Tasks.

Casual Tasks:

    Werden normal ausgeführt.

No Priority Tasks:

    Werden ausgeführt sofern Zeit zu Verfügung steht.
*/

use crossbeam::atomic::AtomicCell;
use crossbeam::deque::Injector;
use std::{
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};
pub mod task;
use task::Task;

pub struct Worker {
    handle: thread::JoinHandle<()>,
    terminate: Arc<AtomicCell<bool>>,
}

pub struct Threadpool {
    workers: Vec<Worker>,
    pub priority_queue: Arc<Injector<Task>>,
    normal_queues: [Arc<Injector<Task>>; 2],
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

        let normal_task_counter = Arc::new(Mutex::new(0));
        let mut workers = Vec::with_capacity(num_threads);

        for i in 0..num_threads {
            let priority_queue = priority_queue.clone();
            let normal_queues = normal_queues.clone();
            let normal_task_counter = normal_task_counter.clone();

            let terminate = Arc::new(AtomicCell::new(false));
            let thread_terminate = terminate.clone();

            workers.push(Worker {
                handle: thread::spawn(move || {
                    loop {
                        // Always handle ALL priority tasks first
                        while let Some(task) = priority_queue.steal().success() {
                            println!("#{}: executes a priority task", i);
                            task.execute(i);
                        }

                        let mut counter = normal_task_counter.lock().unwrap();

                        // Only handle normal/second queue tasks when no priority tasks exist
                        if *counter < 3 {
                            if let Some(task) = normal_queues[0].steal().success().or_else(|| {
                                *counter = 0;
                                normal_queues[1].steal().success()
                            }) {
                                println!("#{}: executes a first task", i);
                                task.execute(i);
                                *counter += 1;
                                continue;
                            }
                        }
                        if let Some(task) = if let Some(task) = normal_queues[1].steal().success() {
                            *counter = 0;
                            Some(task)
                        } else {
                            normal_queues[0].steal().success()
                        } {
                            println!("#{}: executes a second task", i);
                            task.execute(i);
                            *counter += 1;
                            continue;
                        }
                        if thread_terminate.load() {
                            println!("#{}: terminated", i);
                            break;
                        } else {
                            println!("#{}: sleeps", i);
                            thread::sleep(Duration::from_micros(500));
                        }
                    }
                }),
                terminate,
            });
        }

        Threadpool {
            workers,
            priority_queue,
            normal_queues,
        }
    }
    pub fn add_to_first(&mut self, task: Task) {
        self.normal_queues[0].push(task);
    }
    pub fn add_to_second(&mut self, task: Task) {
        self.normal_queues[1].push(task);
    }
    pub fn drop(&mut self) {
        for worker in self.workers.iter() {
            worker.terminate.store(true);
        }
        while let Some(worker) = self.workers.pop() {
            if let Err(e) = worker.handle.join() {
                println!("thread {} panicked, message: {:?}", self.workers.len(), e)
            };
        }
    }
    pub fn size(&self) -> usize {
        self.workers.len()
    }
}
