use actix::Addr;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::controller::ws::ApprovalWs;

#[derive(actix::Message)]
#[rtype(result = "()")]
pub struct NotificationMessage(pub String);

#[derive(Clone)]
pub struct WsManager {
    inner: Arc<Mutex<HashMap<i64, Addr<ApprovalWs>>>>,
}

impl WsManager {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn add(&self, user_id: i64, addr: Addr<ApprovalWs>) {
        let mut map = self.inner.lock().unwrap();
        map.insert(user_id, addr);
    }

    pub fn remove(&self, user_id: i64) {
        let mut map = self.inner.lock().unwrap();
        map.remove(&user_id);
    }

    pub fn send_to_user(&self, user_id: i64, message: String) -> bool {
        let map = self.inner.lock().unwrap();
        if let Some(addr) = map.get(&user_id) {
            addr.do_send(NotificationMessage(message));
            true
        } else {
            false
        }
    }

    pub fn online_users(&self) -> Vec<i64> {
        let map = self.inner.lock().unwrap();
        map.keys().cloned().collect()
    }
}