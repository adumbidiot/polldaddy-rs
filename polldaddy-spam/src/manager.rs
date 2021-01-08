pub mod worker_manager;

pub use crate::manager::worker_manager::{
    WorkerManager,
    WorkerMessage,
};
use free_proxy_list::ProxyInfo;
use parking_lot::RwLock;
use polldaddy::Quiz;
use std::{
    collections::HashSet,
    sync::{
        atomic::{
            AtomicBool,
            Ordering,
        },
        Arc,
    },
    time::Duration,
};

#[derive(Debug)]
pub enum SpawnError {
    DuplicateProxy,
    Shutdown,

    Reqwest(reqwest::Error),
}

pub struct Manager {
    proxy_client: free_proxy_list::Client,
    ip_set: Arc<RwLock<HashSet<String>>>,

    is_shutdown: Arc<AtomicBool>,

    worker_manager: WorkerManager,

    quiz: Arc<Quiz>,
    option: usize,
}

impl Manager {
    pub fn new(quiz: Quiz, option: usize) -> Self {
        Manager {
            proxy_client: Default::default(),
            ip_set: Default::default(),

            is_shutdown: Arc::new(AtomicBool::new(false)),

            worker_manager: WorkerManager::new(),

            quiz: Arc::new(quiz),
            option,
        }
    }

    pub fn len(&self) -> usize {
        self.worker_manager.len()
    }

    pub fn read_message(&self) -> Option<WorkerMessage> {
        self.worker_manager.read_message()
    }

    pub fn shutdown_worker(&self, id: u64) {
        self.worker_manager.shutdown_worker(id);
    }

    pub async fn exit(&self) -> usize {
        self.worker_manager.shutdown();
        self.is_shutdown.store(true, Ordering::SeqCst);
        tokio::time::sleep(Duration::from_secs(20)).await;
        self.len()
    }

    pub async fn fetch_proxies_and_spawn(
        &self,
        timeout: Duration,
    ) -> Option<(Vec<ProxyInfo>, Vec<Result<(), SpawnError>>)> {
        let list = match self.proxy_client.get_list().await {
            Ok(l) => l,
            Err(_e) => {
                return None;
            }
        };

        let valid_list = free_proxy_list::probe(
            list.iter()
                .filter_map(|el| el.as_ref().ok())
                .filter(|el| !self.ip_set.read().contains(&(el.get_url()))),
            timeout,
        )
        .await;

        let good_list = list
            .into_iter()
            .filter_map(|el| el.ok())
            .filter(|el| !self.ip_set.read().contains(&(el.get_url())))
            .zip(valid_list.into_iter())
            .filter(|(_, good)| *good)
            .map(|(i, _)| i)
            .collect::<Vec<_>>();

        let spawn_results = good_list
            .iter()
            .map(|info| self.try_spawn_worker(info))
            .collect();

        Some((good_list, spawn_results))
    }

    pub fn try_spawn_worker(&self, info: &ProxyInfo) -> Result<(), SpawnError> {
        if self.is_shutdown.load(Ordering::SeqCst) {
            return Err(SpawnError::Shutdown);
        }

        let url = info.get_url();
        let ip_set = self.ip_set.clone();

        {
            let mut ip_set_write_lock = ip_set.write();

            if ip_set_write_lock.contains(&url) {
                return Err(SpawnError::DuplicateProxy);
            }

            ip_set_write_lock.insert(url.clone());
        }

        let client = {
            let proxy = reqwest::Proxy::all(&info.get_url()).map_err(SpawnError::Reqwest)?;
            let proxy_client = reqwest::Client::builder()
                .proxy(proxy)
                .timeout(Duration::from_secs(10))
                .build()
                .map_err(SpawnError::Reqwest)?;
            polldaddy::Client::from_reqwest(proxy_client)
        };

        let quiz = self.quiz.clone();
        let option = self.option;

        let worker = self.worker_manager.spawn();

        let _handle = tokio::spawn(async move {
            while !worker.should_exit() {
                worker.send_message(client.vote(&quiz, option).await);
                worker.sleep(Duration::from_secs(10)).await;
            }
            worker.cleanup();
            ip_set.write().remove(&url);
        });

        Ok(())
    }
}
