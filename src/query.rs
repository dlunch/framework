use core::future::Future;

use crate::{read_model::ReadModelStore, Result};

pub trait Query {
    type ReadModelStore: ReadModelStore + 'static;
    type Handler: QueryHandler<Self>;
    type Output;
}

pub trait QueryHandler<Q>
where
    Q: Query + ?Sized,
{
    fn handle(
        read_model_store: &Q::ReadModelStore,
        query: Q,
    ) -> impl Future<Output = Result<Q::Output>> + Send;
}
