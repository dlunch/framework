use core::{any::TypeId, future::Future};

use crate::{as_any::AsAny, event::Event, Result};

pub trait ReadModel: Sync + Send + Default
where
    Self: Sized + 'static,
{
    type Event: Event;

    fn apply_event(&mut self, event: &Self::Event) -> Result<()>;
}

pub trait ReadModelStore: Sync + Send + AsAny {
    type ReadModel: ReadModel;

    fn read_model_event_type() -> TypeId
    where
        Self: Sized,
    {
        TypeId::of::<<Self::ReadModel as ReadModel>::Event>()
    }

    // TODO is there any way to avoid type erasure?
    fn to_concrete<S>(&self) -> Option<&S>
    where
        S: ReadModelStore + 'static,
    {
        self.as_any().downcast_ref()
    }

    fn update_read_model<E>(&self, id: u64, events: &[E]) -> impl Future<Output = Result<()>> + Send
    where
        E: Event + 'static,
    {
        async move {
            let mut read_model = self.read(id).await?.unwrap_or_default();

            for e in events {
                let e = e
                    .as_any()
                    .downcast_ref::<<Self::ReadModel as ReadModel>::Event>()
                    .unwrap();
                read_model.apply_event(e)?;
            }

            self.save(id, &read_model).await?;

            Ok(())
        }
    }

    fn read(&self, id: u64) -> impl Future<Output = Result<Option<Self::ReadModel>>> + Send;
    fn save(
        &self,
        id: u64,
        read_model: &Self::ReadModel,
    ) -> impl Future<Output = Result<()>> + Send;
}

pub trait ReadModelStores {
    fn find<S>(&self) -> Option<&S>
    where
        S: ReadModelStore + 'static;

    fn update_read_model<E>(
        &self,
        id: u64,
        events: &[E],
    ) -> impl Future<Output = Result<()>> + Send
    where
        E: Event + 'static;
}

// TODO macro..
impl ReadModelStores for () {
    fn find<S>(&self) -> Option<&S>
    where
        S: ReadModelStore + 'static,
    {
        None
    }

    async fn update_read_model<E>(&self, _id: u64, _events: &[E]) -> Result<()>
    where
        E: Event + 'static,
    {
        Ok(())
    }
}

impl<S1> ReadModelStores for (S1,)
where
    S1: ReadModelStore + 'static,
{
    fn find<S>(&self) -> Option<&S>
    where
        S: ReadModelStore + 'static,
    {
        if TypeId::of::<S>() == TypeId::of::<S1>() {
            self.0.to_concrete::<S>()
        } else {
            None
        }
    }

    async fn update_read_model<E>(&self, id: u64, events: &[E]) -> Result<()>
    where
        E: Event + 'static,
    {
        if TypeId::of::<E>() == S1::read_model_event_type() {
            self.0.update_read_model(id, events).await?;
        }

        Ok(())
    }
}

impl<S1, S2> ReadModelStores for (S1, S2)
where
    S1: ReadModelStore + 'static,
    S2: ReadModelStore + 'static,
{
    fn find<S>(&self) -> Option<&S>
    where
        S: ReadModelStore + 'static,
    {
        if TypeId::of::<S>() == TypeId::of::<S1>() {
            self.0.to_concrete::<S>()
        } else if TypeId::of::<S>() == TypeId::of::<S2>() {
            self.1.to_concrete::<S>()
        } else {
            None
        }
    }

    async fn update_read_model<E>(&self, id: u64, events: &[E]) -> Result<()>
    where
        E: Event + 'static,
    {
        if TypeId::of::<E>() == S1::read_model_event_type() {
            self.0.update_read_model(id, events).await?;
        }
        if TypeId::of::<E>() == S2::read_model_event_type() {
            self.1.update_read_model(id, events).await?;
        }

        Ok(())
    }
}

impl<S1, S2, S3> ReadModelStores for (S1, S2, S3)
where
    S1: ReadModelStore + 'static,
    S2: ReadModelStore + 'static,
    S3: ReadModelStore + 'static,
{
    fn find<S>(&self) -> Option<&S>
    where
        S: ReadModelStore + 'static,
    {
        if TypeId::of::<S>() == TypeId::of::<S1>() {
            self.0.to_concrete::<S>()
        } else if TypeId::of::<S>() == TypeId::of::<S2>() {
            self.1.to_concrete::<S>()
        } else if TypeId::of::<S>() == TypeId::of::<S3>() {
            self.2.to_concrete::<S>()
        } else {
            None
        }
    }

    async fn update_read_model<E>(&self, id: u64, events: &[E]) -> Result<()>
    where
        E: Event + 'static,
    {
        if TypeId::of::<E>() == S1::read_model_event_type() {
            self.0.update_read_model(id, events).await?;
        }
        if TypeId::of::<E>() == S2::read_model_event_type() {
            self.1.update_read_model(id, events).await?;
        }
        if TypeId::of::<E>() == S3::read_model_event_type() {
            self.2.update_read_model(id, events).await?;
        }

        Ok(())
    }
}
