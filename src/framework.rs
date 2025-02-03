use crate::{
    aggregate::Aggregate,
    command::Command,
    error::FrameworkError,
    event::{EventStore, EventTypeId},
    event_listener::EventListener,
    query::{Query, QueryHandler},
    read_model::ReadModelStores,
    repository::AggregateRepository,
    snapshot::SnapshotStore,
    Result,
};

pub struct Framework<E, S, R>
where
    E: EventStore + 'static,
    S: SnapshotStore + 'static,
    R: ReadModelStores,
{
    event_store: E,
    snapshot_store: S,
    read_model_stores: R,
    event_listener: EventListener,
}

impl<E, S, R> Framework<E, S, R>
where
    E: EventStore + 'static,
    S: SnapshotStore + 'static,
    R: ReadModelStores,
{
    pub fn new(event_store: E, snapshot_store: S, read_model_stores: R) -> Self {
        Self {
            event_store,
            snapshot_store,
            read_model_stores,
            event_listener: EventListener::new(),
        }
    }

    pub async fn command<C>(&self, command: C) -> Result<()>
    where
        C: Command,
    {
        let aggregate_id = command.aggregate_id();

        let repository = AggregateRepository::new(&self.event_store, &self.snapshot_store);

        let aggregate: C::Aggregate = repository.read(command.aggregate_id()).await?;

        let events = aggregate.handle(command)?;

        repository.save(aggregate_id, &events).await?;

        self.event_listener
            .handle_events::<C::Aggregate, R>(&self.read_model_stores, aggregate_id, events)
            .await?;

        Ok(())
    }

    pub async fn query<Q>(&self, query: Q) -> Result<Q::Output>
    where
        Q: Query + 'static,
    {
        let store = self.read_model_stores.find::<Q::ReadModelStore>();

        let Some(store) = store else {
            return Err(FrameworkError::NoSuchReadModelStore);
        };

        Q::Handler::handle(store, query).await
    }

    pub fn register_event_callback<F>(&mut self, event_type_id: EventTypeId, callback: F)
    where
        F: Fn(&dyn crate::Event) -> Result<()> + Send + Sync + 'static,
    {
        self.event_listener
            .register_callback(event_type_id, callback)
    }
}
