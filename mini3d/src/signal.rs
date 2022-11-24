use std::{collections::{HashMap, VecDeque}, any::Any};

use anyhow::{Result, anyhow, Context};
use serde::{Serializer, Deserializer};

use crate::uid::UID;

struct SignalQueue<S> {
    signals: VecDeque<S>,
}

impl<S> SignalQueue<S> {
    fn new() -> Self {
        Self { signals: Default::default() }
    }
}

trait AnySignalQueue: Any {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn clear(&mut self);
}

impl<S: 'static> AnySignalQueue for SignalQueue<S> {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut (dyn Any + 'static) { self }
    fn clear(&mut self) {
        self.signals.clear();
    }
}

#[derive(Default)]
pub struct SignalManager {
    queues: HashMap<UID, Box<dyn AnySignalQueue>>,
}

impl SignalManager {

    pub(crate) fn cleanup(&mut self) {
        for queue in self.queues.values_mut() {
            queue.clear();
        }
    }

    pub(crate) fn save_state<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_unit()
    }

    pub(crate) fn load_state<'de, D: Deserializer<'de>>(&mut self, _deserializer: D) -> Result<(), D::Error> {
        Ok(())
    }

    pub fn register<S: 'static>(&mut self, name: &str) -> Result<()> {
        let uid = UID::new(name);
        if self.queues.contains_key(&uid) { return Err(anyhow!("Signal '{}' already exists", name)); }
        self.queues.insert(uid, Box::new(SignalQueue::<S>::new()));
        Ok(())
    }

    pub fn emit<S: 'static>(&mut self, uid: UID, data: S) -> Result<()> {
        self.queues.get_mut(&uid).with_context(|| "Signal not found")?
            .as_any_mut().downcast_mut::<SignalQueue<S>>().with_context(|| "Invalid signal type")?
            .signals.push_back(data);
        Ok(())
    }

    pub fn iter<S: 'static>(&'_ self, uid: UID) -> Result<impl Iterator<Item = &'_ S>> {
        Ok(self.queues.get(&uid).with_context(|| "Signal not found")?
            .as_any().downcast_ref::<SignalQueue<S>>().with_context(|| "Invalid signal type")?
            .signals.iter())
    }
}