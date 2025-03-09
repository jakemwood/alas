use zbus::zvariant::{ OwnedValue, Value };

pub fn value_to_string(owned_value: OwnedValue) -> Result<String, String> {
    let value = Value::from(owned_value);
    match value {
        Value::Str(s) => Ok(s.to_string()),
        _ => Err("Value is not a string".to_string()),
    }
}
