To add an event, copy ping.rs and give it your name eg "epic.rs"
then import epic.rs into mod.rs, then add it to the networkevent enum, then to the action for networkevent

you should have only created (and filled) one file, and modified mod.rs.


```
use serde::{Serialize, Deserialize};
use crate::handlers::{Handle, NetworkEvent};
use crate::connection::ConnectionLogic;

#[derive(Serialize, Deserialize, Debug)]
pub struct Epic {}
impl Handle for Epic {
    async fn action(&self, connection: &mut ConnectionLogic) {
        todo!()
    }
}

```

```
pub mod epic
...
pub enum NetworkEvent {
    Epic(epic::Epic)
}
...
NetworkEvent::Epic(epic) => epic.action(connection).await
```