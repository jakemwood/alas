# Basic D-Bus Type Codes

## s: String

Represents a UTF-8 encoded string.

Example: `"Modem Manufacturer"`

## u: Unsigned Integer (32-bit)

A 32-bit unsigned integer (range: 0 to 4,294,967,295).

Example: `42`

## i: Signed Integer (32-bit)

A 32-bit signed integer (range: -2,147,483,648 to 2,147,483,647).

Example: `-1`, `0`, `100`

## y: Unsigned Integer (8-bit)

## b: Boolean

Represents a boolean value (true or false).

Example: `true`

## o: Object Path

A D-Bus object path string that uniquely identifies an object.

Example: `"/org/freedesktop/ModemManager1/Modem/0"`

## v: Variant

`Any` type, use `Value` in Zbus

# Composite Type Codes

## a: Array

When prefixed to another type code, it denotes an array of that type.

Examples:

`as`: Array of strings (e.g., ["/dev/ttyUSB0", "/dev/cdc-wdm0"])

`au`: Array of unsigned integers (e.g., [1, 2, 3])

## ( ... ): Struct

Encloses multiple type codes to represent a structured data type with multiple fields.

Example:

`(ub)`: A struct containing a u (unsigned integer) and a b (boolean), e.g., (75, true)

## a{ ... }: Dictionary (Map)

Represents an associative array (map) of key-value pairs, where keys and values are specified types.

Example:

`a{uu}`: A dictionary with unsigned integer keys and unsigned integer values, e.g., {1: 3, 2: 5}
