use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::thread;

use threadpool::ThreadPool;

use crate::service::{ServiceApi, ServiceInitializer, Context};

pub struct TaskContext {
    is_interrupted: AtomicBool,
}

impl TaskContext {
    fn new() -> Self {
        Self {
            is_interrupted: AtomicBool::new(false),
        }
    }
    
    fn stop(&self) {
        self.is_interrupted.store(true, Ordering::Relaxed);
    }

    pub fn is_interrupted(&self) -> bool {
        self.is_interrupted.load(Ordering::Relaxed)
    }
}

pub struct TaskManager {
    pool: Mutex<ThreadPool>,
    tasks: RwLock<Vec<Arc<TaskContext>>>,
}

impl ServiceApi for TaskManager {
    fn stop(&self) {
        let tasks = self.tasks.read().unwrap();
        for task in tasks.iter() {
            task.stop();
        }
    }
}

impl ServiceInitializer for TaskManager {
    fn initialize(_: &Context) -> Arc<Self> {
        Arc::new(TaskManager {
            pool: Mutex::new(ThreadPool::new(4)),
            tasks: RwLock::default(),
        })
    }
}

impl TaskManager {
    pub fn run_instant_task<F>(&self, job: F) where
        F: Fn(&TaskContext) + Send + Sync + 'static
    {
        self.pool.lock().unwrap().execute(move || {
            let task_context = TaskContext::new();
            job(&task_context);
        });
    }

    pub fn run<F>(&self, job: F) where
        F: FnOnce(Arc<TaskContext>) + Send + 'static
    {
        let task_context = Arc::new(TaskContext::new());

        let mut tasks = self.tasks.write().unwrap();
        tasks.push(task_context.clone());

        thread::spawn(move || {
            job(task_context);
        });
    }
}
