import dcmfx_core/data_set
import dcmfx_p10
import gleam/option.{None}

const input_file = "../../example.dcm"

pub fn main() {
  let assert Ok(ds) = dcmfx_p10.read_file(input_file)
  data_set.print(ds, None)
}
