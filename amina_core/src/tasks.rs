use std::sync::{Arc, Mutex, RwLock};
use std::thread;

use threadpool::ThreadPool;

use crate::service::{ServiceApi, ServiceInitializer, Context};

pub struct TaskFeedback {

}

pub trait Task: Send + Sync {
    fn run(&self);
    fn stop(&self);
}

struct TaskContext {
    task: Box<dyn Task>,
}

pub struct TaskManager {
    pool: Mutex<ThreadPool>,
    tasks: RwLock<Vec<Arc<TaskContext>>>,
}

impl ServiceApi for TaskManager {
    fn stop(&self) {
        let tasks = self.tasks.read().unwrap();
        for task in tasks.iter() {
            task.task.stop();
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
        F: Fn(&TaskFeedback) + Send + Sync + 'static
    {
        self.pool.lock().unwrap().execute(move || {
            let feedback = TaskFeedback { };
            job(&feedback);
        });
    }

    pub fn run_generic(&self, task: Box<dyn Task>) {
        let task_context = Arc::new(TaskContext {
            task,
        });

        let mut tasks = self.tasks.write().unwrap();
        tasks.push(task_context.clone());

        thread::spawn(move || {
            task_context.task.run();
        });
    }

}
