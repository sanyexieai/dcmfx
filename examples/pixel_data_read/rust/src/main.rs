use dcmfx::core::*;
use dcmfx::p10::*;
use dcmfx::pixel_data::*;

const INPUT_FILE: &str = "../../example.dcm";

pub fn main() {
    let ds = DataSet::read_p10_file(INPUT_FILE).unwrap();
    let (_vr, frames) = ds.get_pixel_data().unwrap();

    for frame in frames {
        let frame_size =
            frame.iter().fold(0, |acc, bytes| acc + bytes.len());

        println!("Frame with size: {}", frame_size);
    }
}
