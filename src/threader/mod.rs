/* threading goals

verschiedene Arten von Tasks:

Priority Tasks:

    Werden versucht sofort auszuführen. Ignoriert alle anderen Tasks.

First Tasks:

    Werden normal ausgeführt.

Second Tasks:

    Werden ausgeführt sofern Zeit zu Verfügung steht.
*/
use crossbeam::{
    atomic::AtomicCell,
    channel::{bounded, Receiver, Sender},
    deque::Injector,
};
use std::{
    sync::Arc,
    thread,
    time::{Duration, Instant},
};
pub mod task;
use task::Task;

#[derive(Debug)]
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
        for i in 0..num_threads.unwrap_or_else(|| num_cpus::get()) {
            let priority_queue = self.priority_queue.clone();
            let normal_queues = self.normal_queues.clone();

            let wake_call = self.wake_call.clone();
            let sleeping = self.sleeping.clone();

            let Ok(join_handle) = thread::Builder::new()
                .name(format!("{}", i))
                .spawn(move || {
                    let mut counter = 0;
                    let mut poll = 0_usize;
                    loop {
                        // Always handle ALL priority tasks first
                        while let Some(task) = priority_queue.steal().success() {
                            task.run();
                        }

                        // Only handle normal/second queue tasks when no priority tasks exist
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
                            task.run();
                            counter += 1;
                        }
                        if poll < 2 {
                            poll += 1;
                        } else {
                            poll = 0;
                            sleeping.fetch_add(1);
                            match wake_call.recv() {
                                Ok(false) | Err(_) => break,
                                Ok(true) => {
                                    sleeping.fetch_sub(1);
                                }
                            }
                        }
                    }
                    sleeping.fetch_sub(1);
                })
            else {
                println!("thread couldnt been spawned");
                continue;
            };

            self.workers.push(join_handle);
        }
    }
    pub fn update(&mut self) {
        if self.last_update.elapsed().as_nanos() >= 500_000_000 {
            let available_threads = num_cpus::get() as i64 - self.sleeping.load() as i64;

            if available_threads > 0 {
                self.launch(Some(available_threads as usize));
            } else if available_threads < 0 {
                for _ in 0..self.workers.len().min(-available_threads as usize) {
                    let _ = self.waker.send(false);
                }
            }
            self.last_update = Instant::now();
        }
    }
    pub fn add_to_first(&mut self, task: Task) {
        self.normal_queues[0].push(task);
        let _ = self.waker.try_send(true);
    }
    pub fn add_to_second(&mut self, task: Task) {
        self.normal_queues[1].push(task);
        let _ = self.waker.try_send(true);
    }
    pub fn add_priority(&mut self, task: Task) {
        self.priority_queue.push(task);
        let _ = self.waker.try_send(true);
    }
    pub fn dynamic_priority<F>(&mut self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.priority_queue.push(Task::new_dynamic(f));
        let _ = self.waker.try_send(true);
    }
    pub fn drop(self) {
        for _ in 0..self.workers.len() {
            if let Err(..) = self.waker.send_timeout(false, Duration::from_micros(300)) {
                return;
            }
        }
        for worker in self.workers {
            let _ = worker.join();
        }
    }
    fn wake_call(&mut self) {
        let _ = self.waker.try_send(true);
    }
    fn terminator_call(&mut self) {
        let _ = self.waker.send(false);
    }
}
