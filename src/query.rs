use alloc::boxed::Box;

use crate::{read_model::ReadModelStore, Result};

pub trait Query: Sync + Send {
    type ReadModelStore: ReadModelStore + 'static;
    type Handler: QueryHandler<Self>;
    type Output;
}

#[async_trait::async_trait]
pub trait QueryHandler<Q>: Sync + Send
where
    Q: Query + ?Sized,
{
    async fn handle(read_model_store: &Q::ReadModelStore, query: Q) -> Result<Q::Output>;
}
