use alloc::boxed::Box;

use crate::{
    read_model::{ReadModel, ReadModelStore},
    Result,
};

pub trait Query: Sync + Send {
    type ReadModel: ReadModel + 'static;
    type Handler: QueryHandler<Self>;
    type Output;
}

#[async_trait::async_trait]
pub trait QueryHandler<Q>: Sync + Send
where
    Q: Query + ?Sized,
{
    async fn handle<S>(read_model_store: &S, query: Q) -> Result<Q::Output>
    where
        S: ReadModelStore<ReadModel = Q::ReadModel>;
}
