use core::future::Future;

use crate::{read_model::ReadModelStore, Result};

pub trait Query {
    type Handler: QueryHandler<Self>;
}

pub trait QueryHandler<Q>
where
    Q: Query + ?Sized,
{
    type ReadModelStore: ReadModelStore + 'static;
    type Output;

    fn handle(
        read_model_store: &Self::ReadModelStore,
        query: Q,
    ) -> impl Future<Output = Result<Self::Output>> + Send;
}
