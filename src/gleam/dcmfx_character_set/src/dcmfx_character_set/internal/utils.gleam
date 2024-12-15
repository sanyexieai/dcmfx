import gleam/string

/// Returns the codepoint for the replacement character '�' that is emitted when
/// invalid string data is encountered.
///
pub fn replacement_character() -> UtfCodepoint {
  let assert Ok(codepoint) = string.utf_codepoint(0xFFFD)
  codepoint
}

/// Converts an integer codepoint value to a `UtfCodepoint`. The replacement
/// character '�' is returned if the integer is not a valid codepoint.
///
pub fn int_to_codepoint(i: Int) -> UtfCodepoint {
  case string.utf_codepoint(i) {
    Ok(codepoint) -> codepoint
    Error(Nil) -> replacement_character()
  }
}
