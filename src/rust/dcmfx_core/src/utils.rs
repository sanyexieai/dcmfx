/// Removes all whitespace from the end of the passed string. Whitespace is
/// defined as the following Unicode codepoints: U+0000, U+0009, U+000A, U+000D,
/// U+0020.
///
pub fn trim_end_whitespace(s: &str) -> &str {
  s.trim_end_matches(['\0', '\u{0009}', '\u{000A}', '\u{000D}', '\u{0020}'])
}

/// Inspects a byte slice in hexadecimal, e.g. `[1A 2B 3C 4D]`. If the number of
/// bytes in the slice exceeds `max_length` then not all bytes will be
/// shown and a trailing ellipsis will be appended, e.g. `[1A 2B 3C 4D ...]`.
///
pub fn inspect_u8_slice(bytes: &[u8], max_length: usize) -> String {
  let byte_count = std::cmp::min(max_length, bytes.len());

  let s = bytes[0..byte_count]
    .iter()
    .map(|byte| format!("{:02X}", byte))
    .collect::<Vec<_>>()
    .join(" ");

  if byte_count == bytes.len() {
    format!("[{}]", s)
  } else {
    format!("[{} ...]", s)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn inspect_u8_slice_test() {
    assert_eq!(
      inspect_u8_slice(&[0xD1, 0x96, 0x33], 100),
      "[D1 96 33]".to_string()
    );

    assert_eq!(
      inspect_u8_slice(&[0xD1, 0x96, 0x33, 0x44], 3),
      "[D1 96 33 ...]".to_string()
    );
  }
}
