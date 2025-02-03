use alloc::{boxed::Box, vec, vec::Vec};
use core::any::TypeId;

use crate::{as_any::AsAny, event::Event, Result};

pub trait ReadModel: Sync + Send + Default
where
    Self: Sized + 'static,
{
    type Store: ReadModelStore<ReadModel = Self>;
    type Event: Event;

    fn apply_event(&mut self, event: &Self::Event) -> Result<()>;
}

#[async_trait::async_trait]
pub trait ReadModelStore: Sync + Send + AsAny {
    type ReadModel: ReadModel;

    fn read_model_type() -> TypeId
    where
        Self: Sized,
    {
        TypeId::of::<Self::ReadModel>()
    }

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

    async fn read(&self, id: u64) -> Result<Option<Self::ReadModel>>;
    async fn save(&self, id: u64, read_model: &Self::ReadModel) -> Result<()>;
}

#[async_trait::async_trait]
pub trait ReadModelUpdater: Sync + Send {
    async fn update(&self, id: u64, event: &[Box<dyn Event>]) -> Result<()>;
}

struct ReadModelUpdaterImpl<'a, S>(&'a S)
where
    S: ReadModelStore;

#[async_trait::async_trait]
impl<S> ReadModelUpdater for ReadModelUpdaterImpl<'_, S>
where
    S: ReadModelStore,
{
    // TODO is there any way to avoid type erasure?
    async fn update(&self, id: u64, event: &[Box<dyn Event>]) -> Result<()> {
        let mut read_model = self.0.read(id).await?.unwrap_or_default();

        for e in event {
            let e = e
                .as_ref()
                .as_any()
                .downcast_ref::<<S::ReadModel as ReadModel>::Event>()
                .unwrap();
            read_model.apply_event(e)?;
        }

        self.0.save(id, &read_model).await?;

        Ok(())
    }
}

pub trait ReadModelStores: Sync + Send {
    fn store_for_read_model<R>(&self) -> Option<&R::Store>
    where
        R: ReadModel + 'static;

    fn updater_for_event<'a, E>(&'a self) -> Vec<Box<dyn ReadModelUpdater + 'a>>
    where
        E: Event + 'static;
}

impl ReadModelStores for () {
    fn store_for_read_model<R>(&self) -> Option<&R::Store>
    where
        R: ReadModel + 'static,
    {
        None
    }

    fn updater_for_event<'a, E>(&'a self) -> Vec<Box<dyn ReadModelUpdater + 'a>>
    where
        E: Event + 'static,
    {
        Vec::new()
    }
}

impl<S1> ReadModelStores for (S1,)
where
    S1: ReadModelStore + 'static,
{
    fn store_for_read_model<R>(&self) -> Option<&R::Store>
    where
        R: ReadModel + 'static,
    {
        if TypeId::of::<R>() == S1::read_model_type() {
            self.0.to_concrete::<R::Store>()
        } else {
            None
        }
    }

    fn updater_for_event<'a, E>(&'a self) -> Vec<Box<dyn ReadModelUpdater + 'a>>
    where
        E: Event + 'static,
    {
        if TypeId::of::<E>() == S1::read_model_event_type() {
            vec![Box::new(ReadModelUpdaterImpl(&self.0))]
        } else {
            Vec::new()
        }
    }
}

impl<S1, S2> ReadModelStores for (S1, S2)
where
    S1: ReadModelStore + 'static,
    S2: ReadModelStore + 'static,
{
    fn store_for_read_model<R>(&self) -> Option<&R::Store>
    where
        R: ReadModel + 'static,
    {
        if TypeId::of::<R>() == S1::read_model_type() {
            self.0.to_concrete::<R::Store>()
        } else if TypeId::of::<R>() == S2::read_model_type() {
            self.1.to_concrete::<R::Store>()
        } else {
            None
        }
    }

    fn updater_for_event<'a, E>(&'a self) -> Vec<Box<dyn ReadModelUpdater + 'a>>
    where
        E: Event + 'static,
    {
        let mut updaters: Vec<Box<dyn ReadModelUpdater + 'a>> = Vec::new();

        if TypeId::of::<E>() == S1::read_model_event_type() {
            updaters.push(Box::new(ReadModelUpdaterImpl(&self.0)));
        }

        if TypeId::of::<E>() == S2::read_model_event_type() {
            updaters.push(Box::new(ReadModelUpdaterImpl(&self.1)));
        }

        updaters
    }
}

impl<S1, S2, S3> ReadModelStores for (S1, S2, S3)
where
    S1: ReadModelStore + 'static,
    S2: ReadModelStore + 'static,
    S3: ReadModelStore + 'static,
{
    fn store_for_read_model<R>(&self) -> Option<&R::Store>
    where
        R: ReadModel + 'static,
    {
        if TypeId::of::<R>() == S1::read_model_type() {
            self.0.to_concrete::<R::Store>()
        } else if TypeId::of::<R>() == S2::read_model_type() {
            self.1.to_concrete::<R::Store>()
        } else if TypeId::of::<R>() == S3::read_model_type() {
            self.2.to_concrete::<R::Store>()
        } else {
            None
        }
    }

    fn updater_for_event<'a, E>(&'a self) -> Vec<Box<dyn ReadModelUpdater + 'a>>
    where
        E: Event + 'static,
    {
        let mut updaters: Vec<Box<dyn ReadModelUpdater + 'a>> = Vec::new();

        if TypeId::of::<E>() == S1::read_model_event_type() {
            updaters.push(Box::new(ReadModelUpdaterImpl(&self.0)));
        }

        if TypeId::of::<E>() == S2::read_model_event_type() {
            updaters.push(Box::new(ReadModelUpdaterImpl(&self.1)));
        }

        if TypeId::of::<E>() == S3::read_model_event_type() {
            updaters.push(Box::new(ReadModelUpdaterImpl(&self.2)));
        }

        updaters
    }
}

// TODO macro..
