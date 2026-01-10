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
    /*
    typedef enum {
        MM_MODEM_STATE_FAILED        = -1,
        MM_MODEM_STATE_UNKNOWN       = 0,
        MM_MODEM_STATE_INITIALIZING  = 1,
        MM_MODEM_STATE_LOCKED        = 2,
        MM_MODEM_STATE_DISABLED      = 3,
        MM_MODEM_STATE_DISABLING     = 4,
        MM_MODEM_STATE_ENABLING      = 5,
        MM_MODEM_STATE_ENABLED       = 6,
        MM_MODEM_STATE_SEARCHING     = 7,
        MM_MODEM_STATE_REGISTERED    = 8,
        MM_MODEM_STATE_DISCONNECTING = 9,
        MM_MODEM_STATE_CONNECTING    = 10,
        MM_MODEM_STATE_CONNECTED     = 11
    } MMModemState;*/
    #[zbus(property)]
    fn state(&self) -> Result<i32, zbus::Error>;

    #[zbus(property)]
    fn sim(&self) -> Result<ObjectPath, zbus::Error>;

    /// EquipmentIdentifier property
    #[zbus(property)]
    fn equipment_identifier(&self) -> zbus::Result<String>;

    #[zbus(property)]
    fn signal_quality(&self) -> Result<(u32, bool), zbus::Error>;

    #[zbus(signal, name = "StateChanged")]
    fn signal_state_changed(&self, old: i32, new: i32, reason: u32) -> zbus::Result<()>;
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
