import dcmfx_core/data_set
import dcmfx_json
import simplifile

const input_file = "../../example.dcm.json"

pub fn main() {
  let assert Ok(json_data) = simplifile.read(input_file)
  let assert Ok(ds) = dcmfx_json.json_to_data_set(json_data)
  data_set.print(ds)
}
