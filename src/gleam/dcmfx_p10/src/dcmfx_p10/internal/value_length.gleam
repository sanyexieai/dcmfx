import gleam/int

/// Describes a Value Length as stored in DICOM P10, which is either a defined
/// length containing a 32-bit `Int` value, or an undefined length that is
/// encoded as `0xFFFFFFFF` in P10 data.
///
pub type ValueLength {
  Defined(length: Int)
  Undefined
}

/// Constructs a new value length from the given `u32` value. `0xFFFFFFFF` is an
/// undefined length, all other values are a defined length.
///
pub fn new(length: Int) -> ValueLength {
  case length {
    0xFFFFFFFF -> Undefined
    _ -> Defined(length)
  }
}

/// Convert a value length to an `Int`. An undefined length is `0xFFFFFFFF`
/// and all defined lengths are just the contained length value.
///
pub fn to_int(value_length: ValueLength) -> Int {
  case value_length {
    Defined(length) -> length
    Undefined -> 0xFFFFFFFF
  }
}

pub const zero = Defined(0)

/// Converts a value length to a string, e.g. "10 bytes", or "UNDEFINED".
///
pub fn to_string(value_length: ValueLength) -> String {
  case value_length {
    Defined(length) -> int.to_string(length) <> " bytes"
    Undefined -> "UNDEFINED"
  }
}
