use dcmfx::core::*;
use dcmfx::p10::*;

const INPUT_FILE: &str = "../../example.dcm";

pub fn main() {
    let ds = DataSet::read_p10_file(INPUT_FILE).unwrap();
    ds.print();
}
