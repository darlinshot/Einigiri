use wayland_client::{
    Connection,
    Dispatch,
    QueueHandle,
    globals::{ registry_queue_init, GlobalListContents },
    protocol::wl_registry,
};

struct WaylandApp;

impl Dispatch<wl_registry::WlRegistry, GlobalListContents> for WaylandApp {
    fn event(
        state: &mut Self,
        proxy: &wl_registry::WlRegistry,
        event: <wl_registry::WlRegistry as wayland_client::Proxy>::Event,
        data: &GlobalListContents,
        conn: &Connection,
        qhandle: &QueueHandle<Self>,
    )
    {
        println!("Holy shit");
    }
}

pub fn connect() -> Connection {
    // Connect to Wayland
    let conn = Connection::connect_to_env()
        .expect("Failed to connect to Wayland.");

    println!("Connected to Wayland.");

    // Discover available protocols
    let (globals, _event_queues) = registry_queue_init::<WaylandApp>(&conn)
    .expect("Failed to initialize Wayland registry");

    // DEBUG Print every initialized protocol
    globals.contents().with_list(
        |list| {
            for protocol in list {
                println!(
                    "{} (version: {})",
                    protocol.interface,
                    protocol.version
                )
            } 
        }
    );

    conn
}
