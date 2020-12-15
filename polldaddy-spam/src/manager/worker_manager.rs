pub mod atomic_id_gen;

use self::atomic_id_gen::AtomicIdGen;
use crossbeam_queue::{
    PopError,
    SegQueue,
};
use futures::FutureExt;
use parking_lot::RwLock;
use polldaddy::{
    PollError,
    VoteResponse,
};
use std::{
    collections::HashMap,
    sync::{
        atomic::{
            AtomicBool,
            Ordering,
        },
        Arc,
    },
    time::Duration,
};

#[derive(Clone)]
pub struct WorkerManager {
    id_gen: AtomicIdGen,
    workers: Arc<RwLock<HashMap<u64, Worker>>>,
    worker_messages: Arc<SegQueue<WorkerMessage>>,
}

impl WorkerManager {
    pub fn new() -> Self {
        WorkerManager {
            id_gen: AtomicIdGen::new(),
            workers: Default::default(),
            worker_messages: Arc::new(SegQueue::new()),
        }
    }

    pub fn spawn(&self) -> Worker {
        let worker = Worker::new(self.clone());
        self.workers.write().insert(worker.id, worker.clone());
        worker
    }

    pub fn shutdown_worker(&self, id: u64) -> Option<()> {
        self.workers.read().get(&id)?.request_shutdown();
        Some(())
    }

    pub fn shutdown(&self) {
        for (_id, worker) in self.workers.read().iter() {
            worker.request_shutdown();
        }

        while let Ok(_msg) = self.read_message() {}
    }

    pub fn cleanup_worker(&self, id: u64) {
        self.workers.write().remove(&id);
    }

    pub fn len(&self) -> usize {
        self.workers.read().len()
    }

    pub fn read_message(&self) -> Result<WorkerMessage, PopError> {
        self.worker_messages.pop()
    }
}

#[derive(Clone)]
pub struct Worker {
    id: u64,
    should_exit: Arc<AtomicBool>,
    cleaned: Arc<AtomicBool>,

    sleep: Arc<RwLock<Option<tokio::sync::oneshot::Sender<()>>>>,
    manager: WorkerManager,
}

impl Worker {
    pub fn new(manager: WorkerManager) -> Self {
        Worker {
            id: manager.id_gen.get_id(),
            should_exit: Arc::new(AtomicBool::new(false)),
            cleaned: Arc::new(AtomicBool::new(false)),
            sleep: Arc::new(RwLock::new(None)),
            manager,
        }
    }

    #[allow(dead_code)]
    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn should_exit(&self) -> bool {
        self.should_exit.load(Ordering::SeqCst)
    }

    pub fn send_message(&self, data: Result<VoteResponse, PollError>) {
        self.manager
            .worker_messages
            .push(WorkerMessage { id: self.id, data })
    }

    pub fn request_shutdown(&self) {
        self.should_exit.store(true, Ordering::SeqCst);
        self.wake();
    }

    pub fn cleanup(self) {
        if self.cleaned.load(Ordering::SeqCst) {
            return;
        }
        self.manager.cleanup_worker(self.id);
        self.manager.id_gen.free_id(self.id); // Stash the handle and ill take your soul
    }

    pub async fn sleep(&self, time: Duration) {
        let (tx, rx) = tokio::sync::oneshot::channel::<()>();
        *self.sleep.write() = Some(tx);
        futures::select! {
            _ = tokio::time::delay_for(time).fuse() => {},
            _ = rx.fuse() => {},
        }
        *self.sleep.write() = None;
    }

    pub fn wake(&self) -> bool {
        let maybe_tx = self.sleep.write().take();
        match maybe_tx {
            Some(tx) => {
                let _ = tx.send(());
                true
            }
            None => false,
        }
    }
}

pub struct WorkerMessage {
    pub id: u64,
    pub data: Result<VoteResponse, PollError>,
}
