use rhai::plugin::*;

use crate::feature::component::script_storage::ScriptStorage;

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub(crate) struct ScriptStorageHandle(usize);

impl From<&mut ScriptStorage> for ScriptStorageHandle {
    fn from(storage: &mut ScriptStorage) -> Self {
        Self::new(storage)
    }
}

impl AsMut<ScriptStorage> for ScriptStorageHandle {
    fn as_mut(&mut self) -> &mut ScriptStorage {
        unsafe { std::mem::transmute(self.0) }
    }
}

impl ScriptStorageHandle {

    fn new(storage: &mut ScriptStorage) -> Self {
        let handle = unsafe { std::mem::transmute(storage) };
        Self(handle)
    }
}

#[export_module]
pub mod rhai_script_storage_api {

    #[rhai_fn(pure)]
    pub(crate) fn get_bool(storage: &mut ScriptStorageHandle, key: &str) -> bool {
        let storage: &mut ScriptStorage = storage.as_mut(); 
        storage.get_bool(key).unwrap()
    }

    #[rhai_fn(pure)]
    pub(crate) fn set_bool(storage: &mut ScriptStorageHandle, key: &str, value: bool) {
        let storage: &mut ScriptStorage = storage.as_mut();
        storage.set_bool(key, value);
    }

    #[rhai_fn(pure)]
    pub(crate) fn get_int(storage: &mut ScriptStorageHandle, key: &str) -> i32 {
        let storage: &mut ScriptStorage = storage.as_mut(); 
        storage.get_int(key).unwrap()
    }

    #[rhai_fn(pure)]
    pub(crate) fn set_int(storage: &mut ScriptStorageHandle, key: &str, value: i32) {
        let storage: &mut ScriptStorage = storage.as_mut();
        storage.set_int(key, value);
    }

    #[rhai_fn(pure)]
    pub(crate) fn get_float(storage: &mut ScriptStorageHandle, key: &str) -> f32 {
        let storage: &mut ScriptStorage = storage.as_mut(); 
        storage.get_float(key).unwrap()
    }

    #[rhai_fn(pure)]
    pub(crate) fn set_float(storage: &mut ScriptStorageHandle, key: &str, value: f32) {
        let storage: &mut ScriptStorage = storage.as_mut();
        storage.set_float(key, value);
    }

    #[rhai_fn(pure)]
    pub(crate) fn get_string(storage: &mut ScriptStorageHandle, key: &str) -> String {
        let storage: &mut ScriptStorage = storage.as_mut(); 
        storage.get_string(key).unwrap()
    }

    #[rhai_fn(pure)]
    pub(crate) fn set_string(storage: &mut ScriptStorageHandle, key: &str, value: String) {
        let storage: &mut ScriptStorage = storage.as_mut();
        storage.set_string(key, value);
    }

    #[rhai_fn(pure)]
    pub(crate) fn list_keys(storage: &mut ScriptStorageHandle, key: &str) -> rhai::Dynamic {
        let storage: &mut ScriptStorage = storage.as_mut();
        storage.list_keys(key).unwrap().cloned().collect::<Vec<String>>().into()
    }
}

