use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::{ select, signal, time };
use tokio::sync::broadcast::Sender;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use zbus::Connection;
use zbus::export::futures_util::StreamExt;
use zbus::fdo::ObjectManagerProxy;
use zbus::zvariant::{ ObjectPath, OwnedObjectPath, Value };
use crate::modem_manager::{ ModemProxy, ModemSimpleProxy };
use crate::network_manager::{ get_all_devices, DeviceProxy, NetworkManagerProxy, StateChangedArgs };
use crate::state::{ AlasMessage, SafeState };
use crate::wifi::AlasWiFiState;

async fn find_cell_device_path(conn: &Connection) -> Option<OwnedObjectPath> {
    const MODEM: u32 = 8;

    let all_devices = get_all_devices(&conn).await;

    for device_path in all_devices {
        let device_proxy = DeviceProxy::new(conn, device_path.clone()).await.expect(
            "No device proxy"
        );
        let device_type = device_proxy.device_type().await.expect("no type");

        if device_type == MODEM {
            println!("Found modem: {:?}", device_path);
            return Some(device_path);
        }
    }
    None
}

async fn find_device(conn: &Connection) -> DeviceProxy {
    let path = find_cell_device_path(&conn).await.expect("Could not find cell path");
    DeviceProxy::new(&conn, path.clone()).await.expect("Could not connect to device")
}

async fn find_modem_device(conn: &Connection) -> Option<ModemProxy> {
    let path = list_modems().await.unwrap();
    let path = path.first();
    if let Some(path) = path {
        let path = path.to_owned();
        Some(ModemProxy::new(&conn, path.clone()).await.expect("Could not connect to device"))
    }
    else {
        None
    }
}

async fn list_modems() -> Result<Vec<OwnedObjectPath>, Box<dyn std::error::Error>> {
    let conn = Connection::system().await.expect("No connection");
    let obj_man_res = zbus::fdo::ObjectManagerProxy
        ::builder(&conn)
        .destination("org.freedesktop.ModemManager1")?
        .path("/org/freedesktop/ModemManager1")?
        .build().await?;

    let managed_objects = obj_man_res.get_managed_objects().await?;
    Ok(
        managed_objects
            .keys()
            .map(|k| k.to_owned())
            .collect::<Vec<_>>()
    )
}

pub async fn get_imei() -> Option<String> {
    let conn = Connection::system().await.expect("No connection");
    let modem = find_modem_device(&conn).await.expect("No modem found");
    Some(modem.equipment_identifier().await.unwrap().to_string())
}

pub async fn connect_to_cellular(apn_name: String) {
    // Establish a D-Bus connection to the system bus
    let connection = Connection::system().await.expect("No D-bus");

    let device_path = find_cell_device_path(&connection).await.expect("Could not find cellular");
    let device = DeviceProxy::new(&connection, device_path.clone()).await.expect(
        "Could not connect to device"
    );

    let interface = device.interface().await.expect("No interface name");

    // Define the GSM connection settings
    let mut gsm_settings = HashMap::new();
    gsm_settings.insert("apn", Value::from(apn_name));

    let mut connection_settings = HashMap::new();
    connection_settings.insert("id", Value::from("1-gsm"));
    connection_settings.insert("type", Value::from("gsm"));
    connection_settings.insert("interface-name", Value::from(interface));
    connection_settings.insert("autoconnect", Value::from(true));

    let mut settings = HashMap::new();
    settings.insert("connection", connection_settings);
    settings.insert("gsm", gsm_settings);

    // Add the new connection
    let nmp = NetworkManagerProxy::new(&connection).await.expect("No proxy");

    nmp.add_and_activate_connection(
        settings,
        &device_path,
        &ObjectPath::from_str_unchecked("/")
    ).await.expect("No add_connection");
}

pub struct CellObserver {
    pub state: SafeState,
    pub sender: Sender<AlasMessage>,
}

impl CellObserver {
    pub fn new(sender: Sender<AlasMessage>, state: &SafeState) -> Self {
        CellObserver {
            state: state.clone(),
            sender,
        }
    }

    pub async fn listen(self: Arc<Self>) -> JoinHandle<()> {
        let cloned_self = self.clone();

        tokio::spawn(async move {
            let connection = Connection::system().await.expect("Could not get bus");
            let mut device = find_modem_device(&connection).await;

            while device.is_none() {
                println!("📲 Sleeping for 60 seconds before looking for cellular modem...");
                select! {
                    _ = tokio::time::sleep(Duration::from_secs(30)) => {
                        device = find_modem_device(&connection).await;
                    },
                    _ = signal::ctrl_c() => {
                        println!("✅ Stopped looking for cell phone");
                        return;
                    }
                }
            }

            let device = device.unwrap();

            let mut state_change_stream = device
                .receive_signal_state_changed().await
                .expect("Could not start stream");

            println!("📲 Starting cellular status loop");
            loop {
                select! {
                    Some(msg) = state_change_stream.next() => {
                        let quality = Self::get_quality().await;

                        let state_change_msg = msg.args().unwrap();
                        let old_state = state_change_msg.old;
                        let new_state = state_change_msg.new;
                        let reason = state_change_msg.reason;

                        self.set_current_state(new_state, quality).await;

                        println!("📲 Old state: {:?} New state: {:?} Reason: {:?}",
                            old_state, new_state, reason);
                    },
                    _ = time::sleep(Duration::from_secs(5)) => {
                        let current_state = CellObserver::get_current_state().await;
                        let quality = Self::get_quality().await;
                        if let Some(current_state) = current_state {
                            self.set_current_state(current_state, quality).await;
                        } else {
                            println!("📲 There is no cellular state!!!");
                        }
                    },
                    _ = signal::ctrl_c() => {
                        break;
                    }
                }
            }

            println!("📲 Exiting cellular loop");
        })
    }

    async fn get_quality() -> u32 {
        let conn = Connection::system().await.expect("Could not connect to D-bus");
        let device = find_modem_device(&conn).await.expect("No modem found");
        device.signal_quality().await.unwrap().0
    }

    async fn get_current_state() -> Option<i32> {
        let conn = Connection::system().await.expect("Could not connect to D-bus");
        let device = find_modem_device(&conn).await.expect("No modem found");
        match device.state().await {
            Ok(state) => { Some(state) }
            Err(err) => {
                println!("There was an error: {:?}", err);
                None
            }
        }
    }

    async fn set_current_state(&self, state: i32, quality: u32) {
        let mut write_state = self.state.write().await;
        let new_state = {
            if state == 11 { AlasWiFiState::Connected } else { AlasWiFiState::Connecting }
        };
        (*write_state).cell_on = new_state == AlasWiFiState::Connected;
        (*write_state).cell_strength = quality;
        let _ = self.sender.send(AlasMessage::CellularStatusChange {
            new_state,
            cellular_strength: quality,
        });
    }
}
