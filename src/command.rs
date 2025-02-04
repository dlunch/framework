use crate::Aggregate;

pub trait Command {
    type Aggregate: Aggregate<Command = Self> + 'static;

    fn aggregate_id(&self) -> u64;
}
