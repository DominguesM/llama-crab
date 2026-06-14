use std::{
    collections::BTreeMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
};

use crate::{
    error::{PluginError, Result},
    models::LoadedModelInfo,
    worker::WorkerHandle,
};

#[derive(Debug, Default)]
pub struct PluginState {
    models: Mutex<BTreeMap<String, LoadedModelEntry>>,
    requests: Mutex<BTreeMap<String, Arc<AtomicBool>>>,
}

#[derive(Debug, Clone)]
pub(crate) struct LoadedModelEntry {
    pub info: LoadedModelInfo,
    pub worker: Option<WorkerHandle>,
}

impl PluginState {
    pub(crate) fn insert_loaded_model(&self, info: LoadedModelInfo, worker: WorkerHandle) {
        self.models.lock().expect("models lock poisoned").insert(
            info.id.clone(),
            LoadedModelEntry {
                info,
                worker: Some(worker),
            },
        );
    }

    pub(crate) fn worker(&self, id: &str) -> Result<WorkerHandle> {
        self.models
            .lock()
            .expect("models lock poisoned")
            .get(id)
            .and_then(|entry| entry.worker.clone())
            .ok_or_else(|| PluginError::model_not_found(id))
    }

    pub fn loaded_model_ids(&self) -> Vec<String> {
        self.models
            .lock()
            .expect("models lock poisoned")
            .keys()
            .cloned()
            .collect()
    }

    pub fn loaded_model_infos(&self) -> Vec<LoadedModelInfo> {
        self.models
            .lock()
            .expect("models lock poisoned")
            .values()
            .map(|entry| entry.info.clone())
            .collect()
    }

    pub fn model_info(&self, id: &str) -> Option<LoadedModelInfo> {
        self.models
            .lock()
            .expect("models lock poisoned")
            .get(id)
            .map(|entry| entry.info.clone())
    }

    pub(crate) fn remove_model(&self, id: &str) -> Result<LoadedModelEntry> {
        self.models
            .lock()
            .expect("models lock poisoned")
            .remove(id)
            .ok_or_else(|| PluginError::model_not_found(id))
    }

    pub(crate) fn insert_request(&self, request_id: String) -> Arc<AtomicBool> {
        let flag = Arc::new(AtomicBool::new(false));
        self.requests
            .lock()
            .expect("requests lock poisoned")
            .insert(request_id, flag.clone());
        flag
    }

    pub(crate) fn cancel_request(&self, request_id: &str) {
        if let Some(flag) = self
            .requests
            .lock()
            .expect("requests lock poisoned")
            .get(request_id)
        {
            flag.store(true, Ordering::Relaxed);
        }
    }

    pub(crate) fn remove_request(&self, request_id: &str) {
        self.requests
            .lock()
            .expect("requests lock poisoned")
            .remove(request_id);
    }

    pub fn insert_model_for_test(&self, info: LoadedModelInfo) {
        self.models
            .lock()
            .expect("models lock poisoned")
            .insert(info.id.clone(), LoadedModelEntry { info, worker: None });
    }

    pub fn remove_model_for_test(&self, id: &str) -> Option<LoadedModelInfo> {
        self.models
            .lock()
            .expect("models lock poisoned")
            .remove(id)
            .map(|entry| entry.info)
    }
}
