mod modem_manager;
mod network_manager;

use zbus::Connection;
use zbus::zvariant::{OwnedValue, Value};
use crate::modem_manager::ModemSimpleProxy;
use crate::network_manager::NetworkManagerProxy;

fn value_to_string(owned_value: OwnedValue) -> Result<String, String> {
    let value = Value::from(owned_value);
    match value {
        Value::Str(s) => Ok(s.to_string()),
        _ => Err("Value is not a string".to_string()),
    }
}

pub async fn do_things() -> Result<(), Box<dyn std::error::Error>> {
    let conn = Connection::system().await?;

    let nmp = NetworkManagerProxy::new(&conn).await?;
    let all_devices = nmp.get_all_devices().await?;
    dbg!(all_devices);

    let obj_man_res = zbus::fdo::ObjectManagerProxy::builder(&conn)
        .destination("org.freedesktop.ModemManager1")?
        .path("/org/freedesktop/ModemManager1")?
        .build()
        .await?;

    let managed_objects = obj_man_res.get_managed_objects().await?;
    let keys = managed_objects.keys().map(|k| k.to_owned()).collect::<Vec<_>>();
    if keys.len() != 1 {
        panic!("More modems than expected!");
    }
    let modem_name = keys.first().unwrap().to_owned();

    let simple = ModemSimpleProxy::new(&conn, modem_name).await?;
    let status = simple.get_status().await?;

    let operator_name = value_to_string(
        status.get("m3gpp-operator-name").expect("No operator name given").to_owned()
    )?;
    dbg!(operator_name);

    // let sim_path = proxy.sim().await?.to_owned();
    // let sim = SIMProxy::new(&conn, &sim_path).await?;
    // dbg!(sim.operator_name().await?);
    Ok(())
}


// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn it_works() {
//         let result = add(2, 2);
//         assert_eq!(result, 4);
//     }
// }
