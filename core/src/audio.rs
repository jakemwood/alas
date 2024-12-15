// for host in cpal::available_hosts() {
//     println!("host: {}", host.name());
// }
//
// let host = cpal::default_host();
// let input_devices = host.input_devices().expect("No input devices");
// println!("default name: {}", host.default_input_device().expect("should have a default?").name().expect("default should have a name?"));
// for input_device in input_devices {
//     let device_name = input_device.name().expect("Device didn't have a name");
//     println!("device name: {}", device_name);
// }
