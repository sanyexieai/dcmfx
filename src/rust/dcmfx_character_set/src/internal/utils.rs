/// The `char` for the replacement character '�' that is emitted when invalid
/// string data is encountered.
///
pub(crate) const REPLACEMENT_CHARACTER: char = '�';

/// Converts an integer codepoint value to a `char`. The replacement character
/// '�' is returned if the integer is not a valid codepoint.
///
pub(crate) fn codepoint_to_char(codepoint: u32) -> char {
  char::from_u32(codepoint).unwrap_or(REPLACEMENT_CHARACTER)
}
