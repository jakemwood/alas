use std::collections::HashMap;
use zbus::proxy;
use zbus::zvariant::{ ObjectPath, OwnedValue };

#[proxy(
    interface = "org.freedesktop.ModemManager1.Modem.Modem3gpp",
    default_service = "org.freedesktop.ModemManager1"
)]
pub trait Modem3GPP {
    #[zbus(property)]
    fn imei(&self) -> Result<String, zbus::Error>;

    #[zbus(property)]
    fn operator_name(&self) -> Result<String, zbus::Error>;
}

#[proxy(
    interface = "org.freedesktop.ModemManager1.Modem.Simple",
    default_service = "org.freedesktop.ModemManager1"
)]
pub trait ModemSimple {
    fn get_status(&self) -> Result<HashMap<String, OwnedValue>, zbus::Error>;
}

#[proxy(
    interface = "org.freedesktop.ModemManager1.Modem",
    default_service = "org.freedesktop.ModemManager1"
)]
pub trait Modem {
    #[zbus(property)]
    fn state(&self) -> Result<i32, zbus::Error>;

    #[zbus(property)]
    fn sim(&self) -> Result<ObjectPath, zbus::Error>;
}

#[proxy(
    interface = "org.freedesktop.ModemManager1.Sim",
    default_service = "org.freedesktop.ModemManager1"
)]
pub trait SIM {
    #[zbus(property)]
    fn active(&self) -> Result<bool, zbus::Error>;

    #[zbus(property)]
    fn sim_identifier(&self) -> Result<String, zbus::Error>;

    #[zbus(property)]
    fn imsi(&self) -> Result<String, zbus::Error>;

    #[zbus(property)]
    fn eid(&self) -> Result<String, zbus::Error>;

    #[zbus(property)]
    fn operator_identifier(&self) -> Result<String, zbus::Error>;

    #[zbus(property)]
    fn operator_name(&self) -> Result<String, zbus::Error>;
}
