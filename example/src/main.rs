use std::{collections::HashMap, sync::Mutex};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use framework::{
    Aggregate, AggregateTypeId, Command, DummySnapshotStore, Event, EventStore, EventTypeId,
    Framework, Query, QueryHandler, ReadModel, ReadModelStore, Result,
};

#[derive(Serialize, Deserialize, Debug)]
enum EmployeeEvent {
    EmployeeCreated {
        version: u32,
        id: u64,
        name: String,
        address: String,
    },
    NameChanged {
        version: u32,
        name: String,
    },
    AddressChanged {
        version: u32,
        address: String,
    },
}

impl Event for EmployeeEvent {
    fn type_id(&self) -> EventTypeId {
        match self {
            EmployeeEvent::EmployeeCreated { .. } => 1,
            EmployeeEvent::NameChanged { .. } => 2,
            EmployeeEvent::AddressChanged { .. } => 3,
        }
    }

    fn version(&self) -> u32 {
        match self {
            EmployeeEvent::EmployeeCreated { version, .. } => *version,
            EmployeeEvent::NameChanged { version, .. } => *version,
            EmployeeEvent::AddressChanged { version, .. } => *version,
        }
    }
}

enum EmployeeCommand {
    CreateEmployee {
        id: u64,
        name: String,
        address: String,
    },
    ChangeName {
        id: u64,
        name: String,
    },
    ChangeAddress {
        id: u64,
        address: String,
    },
}

impl Command for EmployeeCommand {
    type Aggregate = EmployeeAggregate;

    fn aggregate_id(&self) -> u64 {
        match self {
            EmployeeCommand::CreateEmployee { id, .. } => *id,
            EmployeeCommand::ChangeName { id, .. } => *id,
            EmployeeCommand::ChangeAddress { id, .. } => *id,
        }
    }
}

#[derive(Default, Serialize, Deserialize)]
struct EmployeeAggregate {
    id: u64,
    name: String,
    address: String,
    version: u32,
}

impl Aggregate for EmployeeAggregate {
    type Command = EmployeeCommand;
    type Event = EmployeeEvent;

    fn type_id() -> AggregateTypeId
    where
        Self: Sized,
    {
        1
    }

    fn id(&self) -> u64 {
        self.id
    }

    fn version(&self) -> u32 {
        self.version
    }

    fn handle(&self, command: Self::Command) -> Result<Vec<Self::Event>> {
        match command {
            EmployeeCommand::CreateEmployee { id, name, address } => {
                Ok(vec![EmployeeEvent::EmployeeCreated {
                    version: 1,
                    id,
                    name,
                    address,
                }])
            }
            EmployeeCommand::ChangeName { name, .. } => Ok(vec![EmployeeEvent::NameChanged {
                version: self.version + 1,
                name,
            }]),
            EmployeeCommand::ChangeAddress { address, .. } => {
                Ok(vec![EmployeeEvent::AddressChanged {
                    version: self.version + 1,
                    address,
                }])
            }
        }
    }

    fn apply_events(&mut self, events: &[Self::Event]) -> Result<()> {
        for event in events {
            match event {
                EmployeeEvent::EmployeeCreated {
                    version,
                    id,
                    name,
                    address,
                } => {
                    self.version = *version;
                    self.id = *id;
                    self.name = name.clone();
                    self.address = address.clone();
                }
                EmployeeEvent::NameChanged { version, name, .. } => {
                    self.version = *version;
                    self.name = name.clone();
                }
                EmployeeEvent::AddressChanged {
                    version, address, ..
                } => {
                    self.version = *version;
                    self.address = address.clone();
                }
            }
        }

        Ok(())
    }
}

#[derive(Default, Clone, Debug)]
struct EmployeeReadModel {
    id: u64,
    name: String,
    address: String,
}

impl ReadModel for EmployeeReadModel {
    type Event = EmployeeEvent;

    fn apply_event(&mut self, event: &Self::Event) -> Result<()> {
        match event {
            EmployeeEvent::EmployeeCreated {
                id, name, address, ..
            } => {
                self.id = *id;
                self.name = name.clone();
                self.address = address.clone();
            }
            EmployeeEvent::NameChanged { name, .. } => {
                self.name = name.clone();
            }
            EmployeeEvent::AddressChanged { address, .. } => {
                self.address = address.clone();
            }
        }

        Ok(())
    }
}

struct EmployeeQuery {
    id: u64,
}

impl Query for EmployeeQuery {
    type ReadModelStore = ReadModelStoreImpl;
    type Handler = EmployeeQueryHandler;
    type Output = Option<EmployeeReadModel>;
}

struct EmployeeQueryHandler;

#[async_trait::async_trait]
impl QueryHandler<EmployeeQuery> for EmployeeQueryHandler {
    async fn handle(
        read_model_store: &ReadModelStoreImpl,
        query: EmployeeQuery,
    ) -> framework::Result<Option<EmployeeReadModel>> {
        Ok(read_model_store.read(query.id).await?)
    }
}

#[derive(Default)]
struct ReadModelStoreImpl {
    employees: Mutex<HashMap<u64, EmployeeReadModel>>,
}

#[async_trait::async_trait]
impl ReadModelStore for ReadModelStoreImpl {
    type ReadModel = EmployeeReadModel;

    async fn read(&self, id: u64) -> Result<Option<Self::ReadModel>> {
        Ok(self.employees.lock().unwrap().get(&id).cloned())
    }

    async fn save(&self, id: u64, read_model: &Self::ReadModel) -> Result<()> {
        self.employees
            .lock()
            .unwrap()
            .insert(id, read_model.clone());

        Ok(())
    }
}

#[derive(Default)]
struct EventStoreImpl {
    events: Mutex<HashMap<u64, Vec<Value>>>,
}

#[async_trait::async_trait]
impl EventStore for EventStoreImpl {
    async fn read<A>(&self, aggregate_id: u64, from_version: u32) -> Result<Vec<A::Event>>
    where
        A: Aggregate,
    {
        let events = self
            .events
            .lock()
            .unwrap()
            .get(&aggregate_id)
            .cloned()
            .unwrap_or_default();

        Ok(events
            .iter()
            .map(|x| serde_json::from_value::<A::Event>(x.clone()).unwrap())
            .filter(|x| x.version() > from_version)
            .collect())
    }

    async fn save<A>(&self, aggregate_id: u64, events: &[A::Event]) -> Result<()>
    where
        A: Aggregate,
    {
        self.events
            .lock()
            .unwrap()
            .entry(aggregate_id)
            .or_default()
            .extend(events.iter().map(|x| serde_json::to_value(x).unwrap()));

        Ok(())
    }
}

#[tokio::main]
pub async fn main() -> Result<()> {
    let mut framework = Framework::new(
        EventStoreImpl::default(),
        DummySnapshotStore,
        (ReadModelStoreImpl::default(),),
    );

    framework.register_event_callback(1, |x| {
        println!(
            "EmployeeCreated: {:?}",
            x.as_any().downcast_ref::<EmployeeEvent>()
        );

        Ok(())
    });

    framework
        .execute(EmployeeCommand::CreateEmployee {
            id: 1,
            name: "test".into(),
            address: "address".into(),
        })
        .await?;

    let employee = framework.query(EmployeeQuery { id: 1 }).await?.unwrap();
    println!("{:?}", employee);

    framework
        .execute(EmployeeCommand::ChangeName {
            id: 1,
            name: "new name".into(),
        })
        .await?;

    let employee = framework.query(EmployeeQuery { id: 1 }).await?.unwrap();
    println!("{:?}", employee);

    framework
        .execute(EmployeeCommand::ChangeAddress {
            id: 1,
            address: "new address".into(),
        })
        .await?;

    let employee = framework.query(EmployeeQuery { id: 1 }).await?.unwrap();
    println!("{:?}", employee);

    Ok(())
}
