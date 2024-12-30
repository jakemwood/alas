use std::collections::HashMap;
use zbus::zvariant::{OwnedObjectPath, Value};
use zbus::{proxy, Connection, Result};

#[proxy(
    default_service = "org.freedesktop.NetworkManager",
    default_path = "/org/freedesktop/NetworkManager",
    interface = "org.freedesktop.NetworkManager"
)]
pub trait NetworkManager {
    fn get_all_devices(&self) -> Result<Vec<OwnedObjectPath>>;

    fn get_devices(&self) -> Result<Vec<OwnedObjectPath>>;

    #[zbus(property)]
    fn active_connections(&self) -> Result<Vec<OwnedObjectPath>>;

    fn add_and_activate_connection(
        &self,
        connection: std::collections::HashMap<
            &str,
            std::collections::HashMap<&str, zbus::zvariant::Value<'_>>,
        >,
        device: &zbus::zvariant::ObjectPath<'_>,
        specific_object: &zbus::zvariant::ObjectPath<'_>,
    ) -> Result<(OwnedObjectPath, OwnedObjectPath)>;

    fn deactivate_connection(
        &self,
        active_connection: &zbus::zvariant::ObjectPath<'_>,
    ) -> Result<()>;
}

#[proxy(
    default_service = "org.freedesktop.NetworkManager",
    interface = "org.freedesktop.NetworkManager.Device"
)]
pub trait Device {
    #[zbus(property)]
    fn device_type(&self) -> Result<u32>;

    #[zbus(property)]
    fn udi(&self) -> Result<String>;

    #[zbus(property)]
    fn interface(&self) -> Result<String>;

    #[zbus(property)]
    fn ip_interface(&self) -> Result<String>;

    #[zbus(property)]
    fn driver(&self) -> Result<String>;

    #[zbus(property)]
    fn driver_version(&self) -> Result<String>;

    #[zbus(property)]
    fn firmware_version(&self) -> Result<String>;

    #[zbus(property)]
    fn capabilities(&self) -> Result<u32>;

    #[zbus(property)]
    fn ip4_address(&self) -> Result<u32>;

    #[zbus(property)]
    fn state(&self) -> Result<u32>;

    #[zbus(property)]
    fn state_reason(&self) -> Result<(u32, u32)>;

    #[zbus(signal, name = "StateChanged")]
    fn signal_state_changed(&self, new_state: u32, old_state: u32, reason: u32) -> Result<()>;
}

#[proxy(
    default_service = "org.freedesktop.NetworkManager",
    interface = "org.freedesktop.NetworkManager.Device.Wireless"
)]
pub trait WiFiDevice {
    fn get_access_points(&self) -> Result<Vec<OwnedObjectPath>>;
    fn get_all_access_points(&self) -> Result<Vec<OwnedObjectPath>>;
    fn request_scan(&self, options: HashMap<&str, Value<'_>>) -> Result<()>;

    /// ActiveAccessPoint property
    #[zbus(property)]
    fn active_access_point(&self) -> Result<OwnedObjectPath>;
}

#[proxy(
    default_service = "org.freedesktop.NetworkManager",
    interface = "org.freedesktop.NetworkManager.AccessPoint"
)]
pub trait AccessPoint {
    /// Flags property
    #[zbus(property)]
    fn flags(&self) -> zbus::Result<u32>;

    /// Frequency property
    #[zbus(property)]
    fn frequency(&self) -> zbus::Result<u32>;

    /// HwAddress property
    #[zbus(property)]
    fn hw_address(&self) -> zbus::Result<String>;

    /// LastSeen property
    #[zbus(property)]
    fn last_seen(&self) -> zbus::Result<i32>;

    /// MaxBitrate property
    #[zbus(property)]
    fn max_bitrate(&self) -> zbus::Result<u32>;

    /// Mode property
    #[zbus(property)]
    fn mode(&self) -> zbus::Result<u32>;

    /// RsnFlags property
    #[zbus(property)]
    fn rsn_flags(&self) -> zbus::Result<u32>;

    /// Ssid property
    #[zbus(property)]
    fn ssid(&self) -> zbus::Result<Vec<u8>>;

    /// Strength property
    #[zbus(property)]
    fn strength(&self) -> zbus::Result<u8>;

    /// WpaFlags property
    #[zbus(property)]
    fn wpa_flags(&self) -> zbus::Result<u32>;
}

#[proxy(
    default_service = "org.freedesktop.NetworkManager",
    interface = "org.freedesktop.NetworkManager.Settings",
    default_path = "/org/freedesktop/NetworkManager/Settings"
)]
pub trait Settings {
    /// AddConnection method
    fn add_connection(
        &self,
        connection: std::collections::HashMap<
            &str,
            std::collections::HashMap<&str, zbus::zvariant::Value<'_>>,
        >,
    ) -> Result<OwnedObjectPath>;

    /// AddConnection2 method
    fn add_connection2(
        &self,
        settings: std::collections::HashMap<
            &str,
            std::collections::HashMap<&str, &zbus::zvariant::Value<'_>>,
        >,
        flags: u32,
        args: std::collections::HashMap<&str, &zbus::zvariant::Value<'_>>,
    ) -> Result<(
        OwnedObjectPath,
        std::collections::HashMap<String, zbus::zvariant::OwnedValue>,
    )>;

    /// AddConnectionUnsaved method
    fn add_connection_unsaved(
        &self,
        connection: std::collections::HashMap<
            &str,
            std::collections::HashMap<&str, &zbus::zvariant::Value<'_>>,
        >,
    ) -> Result<OwnedObjectPath>;

    /// GetConnectionByUuid method
    fn get_connection_by_uuid(&self, uuid: &str) -> Result<zbus::zvariant::OwnedObjectPath>;

    /// ListConnections method
    fn list_connections(&self) -> zbus::Result<Vec<zbus::zvariant::OwnedObjectPath>>;

    /// LoadConnections method
    fn load_connections(&self, filenames: &[&str]) -> zbus::Result<(bool, Vec<String>)>;

    /// ReloadConnections method
    fn reload_connections(&self) -> zbus::Result<bool>;

    /// SaveHostname method
    fn save_hostname(&self, hostname: &str) -> zbus::Result<()>;

    /// ConnectionRemoved signal
    #[zbus(signal)]
    fn connection_removed(&self, connection: zbus::zvariant::ObjectPath<'_>) -> zbus::Result<()>;

    /// NewConnection signal
    #[zbus(signal)]
    fn new_connection(&self, connection: zbus::zvariant::ObjectPath<'_>) -> zbus::Result<()>;

    /// CanModify property
    #[zbus(property)]
    fn can_modify(&self) -> zbus::Result<bool>;

    /// Connections property
    #[zbus(property)]
    fn connections(&self) -> zbus::Result<Vec<zbus::zvariant::OwnedObjectPath>>;

    /// Hostname property
    #[zbus(property)]
    fn hostname(&self) -> zbus::Result<String>;
}

#[proxy(
    interface = "org.freedesktop.NetworkManager.Connection.Active",
    default_service = "org.freedesktop.NetworkManager"
)]
pub trait ActiveConnection {
    /// StateChanged signal
    // #[zbus(signal)]
    // fn state_changed(&self, state: u32, reason: u32) -> zbus::Result<()>;

    /// Connection property
    #[zbus(property)]
    fn connection(&self) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;

    /// Controller property
    #[zbus(property)]
    fn controller(&self) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;

    /// Default property
    #[zbus(property)]
    fn default(&self) -> zbus::Result<bool>;

    /// Default6 property
    #[zbus(property)]
    fn default6(&self) -> zbus::Result<bool>;

    /// Devices property
    #[zbus(property)]
    fn devices(&self) -> zbus::Result<Vec<zbus::zvariant::OwnedObjectPath>>;

    /// Dhcp4Config property
    #[zbus(property)]
    fn dhcp4_config(&self) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;

    /// Dhcp6Config property
    #[zbus(property)]
    fn dhcp6_config(&self) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;

    /// Id property
    #[zbus(property)]
    fn id(&self) -> zbus::Result<String>;

    /// Ip4Config property
    #[zbus(property)]
    fn ip4_config(&self) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;

    /// Ip6Config property
    #[zbus(property)]
    fn ip6_config(&self) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;

    /// Master property
    #[zbus(property)]
    fn master(&self) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;

    /// SpecificObject property
    #[zbus(property)]
    fn specific_object(&self) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;

    /// State property
    #[zbus(property)]
    fn state(&self) -> zbus::Result<u32>;

    /// StateFlags property
    #[zbus(property)]
    fn state_flags(&self) -> zbus::Result<u32>;

    /// Type property
    #[zbus(property)]
    fn type_(&self) -> zbus::Result<String>;

    /// Uuid property
    #[zbus(property)]
    fn uuid(&self) -> zbus::Result<String>;

    /// Vpn property
    #[zbus(property)]
    fn vpn(&self) -> zbus::Result<bool>;
}

// #[derive(Deserialize, Serialize, Type, PartialEq, Debug)]
// enum NMDeviceState {
//     NmDeviceStateUnknown = 0, // the device's state is unknown
//     NmDeviceStateUnmanaged = 10, // the device is recognized, but not managed by NetworkManager
//     NmDeviceStateUnavailable = 20, //the device is managed by NetworkManager, but is not available for use. Reasons may include the wireless switched off, missing firmware, no ethernet carrier, missing supplicant or modem manager, etc.
//     NmDeviceStateDisconnected = 30, //the device can be activated, but is currently idle and not connected to a network.
//     NmDeviceStatePrepare = 40, //the device is preparing the connection to the network. This may include operations like changing the MAC address, setting physical link properties, and anything else required to connect to the requested network.
//     NmDeviceStateConfig = 50, //the device is connecting to the requested network. This may include operations like associating with the WiFi AP, dialing the modem, connecting to the remote Bluetooth device, etc.
//     NmDeviceStateNeedAuth = 60, // the device requires more information to continue connecting to the requested network. This includes secrets like WiFi passphrases, login passwords, PIN codes, etc.
//     NmDeviceStateIpConfig = 70, // the device is requesting IPv4 and/or IPv6 addresses and routing information from the network.
//     NmDeviceStateIpCheck = 80, // the device is checking whether further action is required for the requested network connection. This may include checking whether only local network access is available, whether a captive portal is blocking access to the Internet, etc.
//     NmDeviceStateSecondaries = 90, //the device is waiting for a secondary connection (like a VPN) which must activated before the device can be activated
//     NmDeviceStateActivated = 100, //the device has a network connection, either local or global.
//     NmDeviceStateDeactivating = 110, // a disconnection from the current network connection was requested, and the device is cleaning up resources used for that connection. The network connection may still be valid.
//     NmDeviceStateFailed = 120, // the device failed to connect to the requested network and is cleaning up the connection request
// }
//
// impl From<u32> for NMDeviceState {
//     fn from(state: u32) -> Self {
//         match state {
//             0 => NMDeviceState::NmDeviceStateUnknown,
//             10 => NMDeviceState::NmDeviceStateUnmanaged,
//             20 => NMDeviceState::NmDeviceStateUnavailable,
//             30 => NMDeviceState::NmDeviceStateDisconnected,
//             40 => NMDeviceState::NmDeviceStatePrepare,
//             50 => NMDeviceState::NmDeviceStateConfig,
//             60 => NMDeviceState::NmDeviceStateNeedAuth,
//             70 => NMDeviceState::NmDeviceStateIpConfig,
//             80 => NMDeviceState::NmDeviceStateIpCheck,
//             90 => NMDeviceState::NmDeviceStateSecondaries,
//             100 => NMDeviceState::NmDeviceStateActivated,
//             110 => NMDeviceState::NmDeviceStateDeactivating,
//             120 => NMDeviceState::NmDeviceStateFailed,
//             _ => NMDeviceState::NmDeviceStateUnknown,
//         }
//     }
// }

pub async fn get_all_devices(conn: &Connection) -> Vec<OwnedObjectPath> {
    let nmp = NetworkManagerProxy::new(&conn).await.expect("Oops");
    nmp.get_devices().await.expect("No devices")
}
