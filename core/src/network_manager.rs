use zbus::zvariant::OwnedObjectPath;
use zbus::{proxy, Error};


#[proxy(
    default_service="org.freedesktop.NetworkManager",
    default_path="/org/freedesktop/NetworkManager",
    interface="org.freedesktop.NetworkManager"
)]
pub trait NetworkManager {
    fn get_all_devices(&self) -> Result<Vec<OwnedObjectPath>, Error>;
}


#[proxy(
    default_service="org.freedesktop.NetworkManager",
    interface="org.freedesktop.NetworkManager.Device",
)]
pub trait Device {
    #[zbus(property)]
    fn udi(&self) -> Result<String, Error>;

    #[zbus(property)]
    fn interface(&self) -> Result<String, Error>;

    #[zbus(property)]
    fn ip_interface(&self) -> Result<String, Error>;

    #[zbus(property)]
    fn driver(&self) -> Result<String, Error>;

    #[zbus(property)]
    fn driver_version(&self) -> Result<String, Error>;

    #[zbus(property)]
    fn firmware_version(&self) -> Result<String, Error>;

    #[zbus(property)]
    fn capabilities(&self) -> Result<u32, Error>;

    #[zbus(property)]
    fn ip4_address(&self) -> Result<u32, Error>;

    #[zbus(property)]
    fn state(&self) -> Result<u32, Error>;

    #[zbus(property)]
    fn state_reason(&self) -> Result<(u32, u32), Error>;
}
