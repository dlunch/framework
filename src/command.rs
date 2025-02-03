use crate::Aggregate;

pub trait Command: Sync + Send {
    type Aggregate: Aggregate<Command = Self> + 'static;

    fn aggregate_id(&self) -> u64;
}
