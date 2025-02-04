use core::future::Future;

use crate::{aggregate::Aggregate, Result};

pub trait SnapshotStore {
    fn read<A>(&self, aggregate_id: u64) -> impl Future<Output = Result<Option<A>>> + Send
    where
        A: Aggregate;
    fn save<A>(&self, aggregate: &A) -> impl Future<Output = Result<()>> + Send
    where
        A: Aggregate;
}

pub struct DummySnapshotStore;

impl SnapshotStore for DummySnapshotStore {
    async fn read<A>(&self, _aggregate_id: u64) -> Result<Option<A>>
    where
        A: Aggregate,
    {
        Ok(None)
    }

    async fn save<A>(&self, _aggregate: &A) -> Result<()>
    where
        A: Aggregate,
    {
        Ok(())
    }
}
