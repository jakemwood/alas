pub mod audio;
pub mod config;
pub mod dropbox;
mod modem_manager;
mod network_manager;
pub mod state;
mod utils;
pub mod wifi;
pub mod cellular;
pub mod redundancy;
pub mod webhook;

use crate::modem_manager::ModemSimpleProxy;
use zbus::Connection;

pub async fn do_things() -> Result<(), Box<dyn std::error::Error>> {
    let conn = Connection::system().await?;

    // Connect to WiFi
    // wifi::disconnect_wifi(&conn).await;
    // wifi::connect_to_wifi(
    //     &conn,
    //     wifi_device.clone(),
    //     access_points
    //         .first()
    //         .expect("no access points found"),
    //     String::from("Southwest Airlines"),
    // )
    // .await;
    // println!("Connected to WiFi?!");

    let obj_man_res = zbus::fdo::ObjectManagerProxy
        ::builder(&conn)
        .destination("org.freedesktop.ModemManager1")?
        .path("/org/freedesktop/ModemManager1")?
        .build().await?;

    let managed_objects = obj_man_res.get_managed_objects().await?;
    let keys = managed_objects
        .keys()
        .map(|k| k.to_owned())
        .collect::<Vec<_>>();
    if keys.len() != 1 {
        panic!("More modems than expected!");
    }
    let modem_name = keys.first().unwrap().to_owned();

    let simple = ModemSimpleProxy::new(&conn, modem_name).await?;
    let status = simple.get_status().await?;

    let operator_name = utils::value_to_string(
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
