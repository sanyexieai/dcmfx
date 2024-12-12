import dcmfx_core/internal/utils
import gleeunit/should

pub fn trim_codepoints_test() {
  utils.trim_ascii("  \n234 ", 0x20)
  |> should.equal("\n234")
}

pub fn trim_end_codepoints_test() {
  utils.trim_ascii_end("\n\n\n 234 \n\n", 0x0A)
  |> should.equal("\n\n\n 234 ")
}
