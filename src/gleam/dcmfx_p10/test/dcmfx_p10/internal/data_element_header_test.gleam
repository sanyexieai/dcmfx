import dcmfx_core/dictionary
import dcmfx_core/value_representation
import dcmfx_p10/internal/data_element_header.{DataElementHeader}
import dcmfx_p10/internal/value_length
import gleam/option.{None, Some}
import gleeunit/should

pub fn to_string_test() {
  DataElementHeader(
    dictionary.patient_age.tag,
    Some(value_representation.AgeString),
    value_length.zero,
  )
  |> data_element_header.to_string
  |> should.equal("(0010,1010) AS Patient's Age")

  DataElementHeader(dictionary.item.tag, None, value_length.zero)
  |> data_element_header.to_string
  |> should.equal("(FFFE,E000)    Item")
}
