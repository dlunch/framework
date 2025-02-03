use core::marker::PhantomData;

use crate::{aggregate::Aggregate, event::EventStore, snapshot::SnapshotStore, Result};

pub struct AggregateRepository<'a, A, E, S>
where
    A: Aggregate,
    E: EventStore,
    S: SnapshotStore,
{
    event_store: &'a E,
    snapshot_store: &'a S,
    _phantom: PhantomData<A>,
}

impl<'a, A, E, S> AggregateRepository<'a, A, E, S>
where
    A: Aggregate,
    E: EventStore,
    S: SnapshotStore,
{
    pub fn new(event_store: &'a E, snapshot_store: &'a S) -> Self {
        Self {
            event_store,
            snapshot_store,
            _phantom: PhantomData,
        }
    }

    pub async fn read(&self, aggregate_id: u64) -> Result<A> {
        let mut aggregate = self
            .snapshot_store
            .read::<A>(aggregate_id)
            .await?
            .unwrap_or_default();

        let events = self
            .event_store
            .read::<A>(aggregate_id, aggregate.version())
            .await?;

        aggregate.apply_events(&events)?;

        Ok(aggregate)
    }

    pub async fn save(&self, aggregate_id: u64, events: &[A::Event]) -> Result<()> {
        self.event_store.save::<A>(aggregate_id, events).await?;

        // update snapshot
        let aggregate = self.read(aggregate_id).await?;
        self.snapshot_store.save(&aggregate).await?;

        Ok(())
    }
}
