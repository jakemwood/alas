use crate::network_manager::get_all_devices;
use crate::network_manager::{
    AccessPointProxy, ActiveConnectionProxy, DeviceProxy, NetworkManagerProxy, StateChangedArgs,
    WiFiDeviceProxy,
};
use crate::state::AlasMessage;
use serde::Serialize;
use std::collections::HashMap;
use tokio::sync::broadcast::Sender;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tokio::{select, signal};
use uuid::Uuid;
use zbus::export::futures_util::StreamExt;
use zbus::zvariant::{ObjectPath, OwnedObjectPath, Value};
use zbus::Connection;

/// Find the device path that is responsible for Wi-Fi.
///
/// Returns the path if one is found.
async fn find_wifi_device_path(conn: &Connection) -> Option<OwnedObjectPath> {
    const WIFI: u32 = 2;

    let all_devices = get_all_devices(&conn).await;

    for device_path in all_devices {
        let device_proxy = DeviceProxy::new(conn, device_path.clone())
            .await
            .expect("No proxy");
        let device_type = device_proxy.device_type().await.expect("No type");

        if device_type == WIFI {
            // We've found it!
            return Some(device_path);
        }
    }
    None
}

/// Get a Z-bus proxy for working with the Wi-Fi device
async fn find_wifi_device(conn: &Connection) -> WiFiDeviceProxy {
    let path = find_wifi_device_path(&conn)
        .await
        .expect("Could not find Wi-Fi device");
    WiFiDeviceProxy::new(conn, path)
        .await
        .expect("No WiFi device proxy")
}

/// Get a Z-bux proxy for working with the Device (with a capital "D") that is responsible for Wi-Fi
async fn get_wifi_device_as_device(conn: &Connection) -> DeviceProxy {
    let path = find_wifi_device_path(&conn)
        .await
        .expect("Could not find Wi-Fi device");
    DeviceProxy::new(conn, path)
        .await
        .expect("No DeviceProxy for Wi-Fi")
}

/// Given a list of paths to Access Points, load the data into a structure that we like
async fn load_access_points(
    conn: &Connection,
    all_access_points: Vec<OwnedObjectPath>,
) -> Vec<WiFiNetwork> {
    let mut results: Vec<WiFiNetwork> = Vec::new();
    for access_point in all_access_points {
        let app = AccessPointProxy::new(&conn, access_point.clone())
            .await
            .expect("No proxy");
        let ssid =
            String::from_utf8(app.ssid().await.expect("No ssid")).expect("Could not convert SSID");

        let security = if app.rsn_flags().await.expect("No security") & 0x100 != 0 {
            "wpa"
        } else {
            "none"
        };

        results.push(WiFiNetwork {
            ssid,
            strength: app.strength().await.expect("No strength"),
            ap_path: access_point.clone(),
            frequency: app.frequency().await.expect("No frequency"),
            security: String::from(security),
        });
    }
    results
}

/// Create a Wi-Fi hotspot
async fn create_wifi_hotspot(conn: &Connection) {
    let mut connection: HashMap<&str, Value> = HashMap::new();
    connection.insert("type", "802-11-wireless".into());
    connection.insert("uuid", Value::from(Uuid::new_v4().to_string())); // Generate a unique UUID
    connection.insert("id", "Alas Config".into());

    let mut wireless: HashMap<&str, Value> = HashMap::new();
    wireless.insert("ssid", "Alas Config".as_bytes().into());
    wireless.insert("mode", "ap".into());
    wireless.insert("band", "bg".into());
    wireless.insert("channel", 6u32.into()); // Choose an appropriate channel

    let mut ipv4: HashMap<&str, Value> = HashMap::new();
    ipv4.insert("method", "shared".into());

    let mut connection_info: HashMap<&str, HashMap<&str, Value>> = HashMap::new();
    connection_info.insert("connection", connection);
    connection_info.insert("802-11-wireless", wireless);
    connection_info.insert("ipv4", ipv4);

    disconnect_wifi(&conn).await;

    let nmp = NetworkManagerProxy::new(&conn).await.expect("No proxy");
    let wifi_device_path = find_wifi_device_path(&conn).await.expect("No Wi-Fi");
    nmp.add_and_activate_connection(
        connection_info,
        &wifi_device_path,
        &ObjectPath::from_str_unchecked("/"),
    )
    .await
    .expect("No add_connection");
}

/// Connect to a Wi-fi access point
async fn connect_to_wifi(
    conn: &Connection,
    wifi_device: OwnedObjectPath,
    access_point_path: String,
    password: Option<String>,
) {
    let access_point = AccessPointProxy::new(&conn, access_point_path)
        .await
        .expect("No access path by this name");
    let ssid_bytes = access_point.ssid().await.expect("No SSID");
    let ssid = String::from_utf8(ssid_bytes.clone()).unwrap();

    let nmp = NetworkManagerProxy::new(&conn).await.expect("No proxy");
    let mut connection_info = HashMap::new();

    let mut s_conn = HashMap::new();
    s_conn.insert("type", Value::from("802-11-wireless"));
    s_conn.insert("uuid", Value::from(Uuid::new_v4().to_string()));
    s_conn.insert("id", Value::from(ssid.clone()));

    let mut s_wifi = HashMap::new();

    s_wifi.insert("ssid", Value::from(ssid_bytes.clone()));
    s_wifi.insert("mode", Value::from("infrastructure"));

    let mut s_wsec = HashMap::new();
    // TODO: figure out what these options are
    if password.is_some() {
        s_wsec.insert("key-mgmt", Value::from("wpa-psk"));
        s_wsec.insert("auth-alg", Value::from("open"));
        s_wsec.insert("psk", Value::from(password.unwrap()));
    }

    let mut s_ip4 = HashMap::new();
    s_ip4.insert("method", Value::from("auto"));

    let mut s_ip6 = HashMap::new();
    s_ip6.insert("method", Value::from("ignore"));

    connection_info.insert("connection", s_conn);
    connection_info.insert("802-11-wireless", s_wifi);
    connection_info.insert("802-11-wireless-security", s_wsec);
    connection_info.insert("ipv4", s_ip4);
    connection_info.insert("ipv6", s_ip6);

    disconnect_wifi(&conn).await;

    nmp.add_and_activate_connection(connection_info, &wifi_device, access_point.inner().path())
        .await
        .expect("No add_connection");
}

/// Disconnect the current Wi-Fi. This will either disconnect from an
/// access point, turn off the hotspot, etc.
async fn disconnect_wifi(conn: &Connection) {
    let nmp = NetworkManagerProxy::new(&conn).await.expect("No proxy");

    let active_connections = nmp.active_connections().await.expect("No connections");

    for connection in active_connections {
        let active_connection = ActiveConnectionProxy::new(&conn, connection.clone())
            .await
            .expect("No proxy");
        if active_connection.type_().await.unwrap() == "802-11-wireless" {
            nmp.deactivate_connection(&connection)
                .await
                .expect("no deactivation");
            return;
        }
    }
}

/******************************************************************
 PUBLIC API
******************************************************************/

/// This structure represents the basics of a Wi-Fi network
#[derive(Debug, Serialize)]
pub struct WiFiNetwork {
    pub ssid: String,
    pub strength: u8,
    pub ap_path: OwnedObjectPath,
    pub frequency: u32,
    pub security: String,
}

/// Get a list of Wi-Fi networks
pub async fn get_wifi_networks() -> Vec<WiFiNetwork> {
    let conn = Connection::system()
        .await
        .expect("Could not connect to D-bus");
    let wifi_device = find_wifi_device(&conn).await;
    wifi_device
        .request_scan(HashMap::new())
        .await
        .expect("did not scan");
    let raw_access_points = wifi_device
        .get_all_access_points()
        .await
        .expect("Needs access points");
    load_access_points(&conn, raw_access_points.clone()).await
}

/// Join a Wi-Fi network, given an Access Point Path and the password, if necessary.
pub async fn join_wifi(path: String, password: Option<String>) {
    let conn = Connection::system()
        .await
        .expect("Could not connect to D-bus");
    let wifi_device_path = find_wifi_device_path(&conn).await.expect("No Wi-Fi");
    connect_to_wifi(&conn, wifi_device_path, path, password).await;
}

pub async fn create_config_hotspot() {
    let conn = Connection::system()
        .await
        .expect("Could not connect to D-bus");
    create_wifi_hotspot(&conn).await;
}

/// WiFiObserver allows us to subscribe to signals from D-bus about the state of Wi-Fi.
pub struct WiFiObserver {
    pub sender: Sender<AlasMessage>,
    pub state: RwLock<Option<u32>>,
}

impl WiFiObserver {
    pub fn new(broadcast: Sender<AlasMessage>) -> Self {
        println!("Starting WiFi server...");
        WiFiObserver {
            sender: broadcast,
            state: RwLock::new(None),
        }
    }

    pub async fn listen_for_wifi_changes(&self) -> JoinHandle<()> {
        let connection = Connection::system().await.expect("Could not get bus");

        let wifi_device_path = find_wifi_device_path(&connection)
            .await
            .expect("Did not find Wi-Fi device path");

        let device_proxy = DeviceProxy::new(&connection, wifi_device_path)
            .await
            .expect("no device proxy");

        let sender = self.sender.clone();
        let current_state = self.refresh_current_wifi_state().await.unwrap();

        tokio::spawn(async move {
            // Get the current state
            println!("Current state is {:?}", current_state);
            match sender.send(AlasMessage::NetworkStatusChange {
                new_state: current_state,
            }) {
                Ok(_) => {}
                Err(err) => {
                    println!("Error sending Wi-Fi state: {:?}", err);
                }
            }

            let mut state_change_stream = device_proxy
                .receive_signal_state_changed()
                .await
                .expect("Could not start stream?");

            println!("Waiting for network changes...");
            loop {
                select! {
                    Some(msg) = state_change_stream.next() => {
                        let args: StateChangedArgs = msg.args().expect("Error parsing message");
                        dbg!(&args);
                        let _ = sender.send(AlasMessage::NetworkStatusChange {
                            new_state: args.new_state,
                        });
                    },
                    _ = signal::ctrl_c() => {
                        break;
                    }
                }
            }
            println!("Exiting the Wi-Fi loop!");
        })
    }

    pub async fn refresh_current_wifi_state(&self) -> Option<u32> {
        println!("getting current state!");
        let conn = Connection::system()
            .await
            .expect("Could not connect to D-bus");
        let wifi_device = get_wifi_device_as_device(&conn).await;
        let mut state = self.state.write().await;
        *state = Some(wifi_device.state().await.unwrap());
        *state
    }

    pub async fn get_state(&self) -> Option<u32> {
        *self.state.read().await
    }
}
