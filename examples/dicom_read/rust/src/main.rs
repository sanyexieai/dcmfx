use dcmfx::core::*;
use dcmfx::p10::*;

const INPUT_FILE: &str = "../../example.dcm";

pub fn main() {
    let ds = DataSet::read_p10_file(INPUT_FILE).unwrap();
    ds.print();

    let patient_id = ds.get_string(registry::PATIENT_ID.tag).unwrap();
    println!("Patient ID: {}", patient_id);

    let study_date = ds.get_date(registry::STUDY_DATE.tag).unwrap();
    println!("Study Date: {:?}", study_date);
}
