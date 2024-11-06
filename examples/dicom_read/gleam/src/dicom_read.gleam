import dcmfx_core/data_set
import dcmfx_core/registry
import dcmfx_p10
import gleam/io
import gleam/string

const input_file = "../../example.dcm"

pub fn main() {
  let assert Ok(ds) = dcmfx_p10.read_file(input_file)
  data_set.print(ds)

  let assert Ok(patient_id) = data_set.get_string(ds, registry.patient_id.tag)
  io.println("Patient ID: " <> patient_id)

  let assert Ok(study_date) = data_set.get_date(ds, registry.study_date.tag)
  io.println("Study Date: " <> string.inspect(study_date))
}
