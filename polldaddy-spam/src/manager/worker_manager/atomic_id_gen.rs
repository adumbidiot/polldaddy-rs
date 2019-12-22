use parking_lot::RwLock;
use std::sync::{
    atomic::{
        AtomicU64,
        Ordering,
    },
    Arc,
};

#[derive(Debug, Default, Clone)]
pub struct AtomicIdGen {
    id: Arc<AtomicU64>,
    free_list: Arc<RwLock<Vec<u64>>>,
}

impl AtomicIdGen {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn get_id(&self) -> u64 {
        let mut free_list = self.free_list.write();
        if let Some(n) = free_list.pop() {
            return n;
        }

        self.id.fetch_add(1, Ordering::SeqCst)
    }

    pub fn free_id(&self, id: u64) {
        self.free_list.write().push(id);
    }
}
