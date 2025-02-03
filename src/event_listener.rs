use alloc::{boxed::Box, collections::BTreeMap, vec::Vec};

use crate::{
    aggregate::Aggregate,
    event::{Event, EventTypeId},
    read_model::ReadModelStores,
    Result,
};

type BoxedEventCallback = Box<dyn Fn(&dyn Event) -> Result<()> + Send + Sync>;

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

    pub async fn handle_events<A, R>(
        &self,
        read_model_stores: &R,
        aggregate_id: u64,
        events: Vec<A::Event>,
    ) -> Result<()>
    where
        A: Aggregate + 'static,
        R: ReadModelStores,
    {
        let boxed_events = events
            .into_iter()
            .map(|e| Box::new(e) as Box<dyn Event>)
            .collect::<Vec<_>>();

        // 1. update read model
        let updaters = read_model_stores.updater_for_event::<A::Event>();

        for updater in &updaters {
            updater.update(aggregate_id, &boxed_events).await?;
        }

        // 2. dispatch callbacks
        for e in &boxed_events {
            if let Some(callback) = self.callbacks.get(&e.type_id()) {
                callback(e.as_ref())?;
            }
        }

        Ok(())
    }

    pub fn register_callback<F>(&mut self, event_type_id: EventTypeId, callback: F)
    where
        F: Fn(&dyn Event) -> Result<()> + Send + Sync + 'static,
    {
        self.callbacks.insert(event_type_id, Box::new(callback));
    }
}
