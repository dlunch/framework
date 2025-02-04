use alloc::{boxed::Box, collections::BTreeMap, vec::Vec};

use crate::{
    aggregate::Aggregate,
    event::{Event, EventTypeId},
    Result,
};

type BoxedEventCallback = Box<dyn Fn(&dyn Event) -> Result<()> + Sync + Send>;

#[derive(Default)]
pub struct EventListener {
    callbacks: BTreeMap<EventTypeId, BoxedEventCallback>,
}

impl EventListener {
    pub fn new() -> Self {
        Self {
            callbacks: BTreeMap::new(),
        }
    }

    pub async fn handle_events<A>(&self, events: Vec<A::Event>) -> Result<()>
    where
        A: Aggregate + 'static,
    {
        for e in &events {
            if let Some(callback) = self.callbacks.get(&e.type_id()) {
                callback(e)?;
            }
        }

        Ok(())
    }

    pub fn register_callback<F>(&mut self, event_type_id: EventTypeId, callback: F)
    where
        F: Fn(&dyn Event) -> Result<()> + Sync + Send + 'static,
    {
        self.callbacks.insert(event_type_id, Box::new(callback));
    }
}
