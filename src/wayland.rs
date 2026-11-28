// LIBRARIES 
use smithay_client_toolkit as sctk;
use wayland_client as wc;

use sctk::{
    delegate_registry,
    delegate_dispatch2,

    registry_handlers,
    registry::{
        RegistryState,
        ProvidesRegistryState,
    }
    
};
use wc::{
    Connection,
    QueueHandle,
    EventQueue,
    globals::{
        registry_queue_init
    },
};

// STRUCTS 
struct WaylandState {
    // List of events, and signals new globals added and old globals removed
    registry_state: RegistryState,

    // Send object's future events to queue
    qh: QueueHandle<WaylandState>
}
pub struct WaylandApp {
    connection: Connection,
    event_queue: EventQueue<WaylandState>,
    state: WaylandState
}

// FUNCTIONS 
fn connect() -> Connection {
    let conn = Connection::connect_to_env()
        .expect("Failed to connect to Wayland.");

    conn
}

// IMPLEMENTATIONS
impl WaylandApp {
    pub fn new() -> Self {
        let conn = connect();
        let (globals, event_queue) = registry_queue_init(&conn)
        .expect("Failed to register Wayland globals.");

        let qh = event_queue.handle();

        #[allow(unused_mut)]
        let mut state = WaylandState {
            registry_state: RegistryState::new(&globals),
            qh: qh
        };

        Self {
            connection: conn,
            event_queue: event_queue,
            state: state
        }
    }
} 

// Tells SCTK WaylandState contains RegistryState
impl ProvidesRegistryState for WaylandState {
    fn registry(&mut self) -> &mut RegistryState {

        // RegistryState is here
        &mut self.registry_state
    }
    registry_handlers![];
}

delegate_registry!(WaylandState);
delegate_dispatch2!(WaylandState);