import dcmfx_p10
import dcmfx_pixel_data
import gleam/bit_array
import gleam/int
import gleam/io
import gleam/list

const input_file = "../../example.dcm"

pub fn main() {
  let assert Ok(ds) = dcmfx_p10.read_file(input_file)
  let assert Ok(#(_vr, frames)) = dcmfx_pixel_data.get_pixel_data(ds)

  frames
  |> list.each(fn(frame_items) {
    let frame_size =
      list.fold(frame_items, 0, fn(acc, bytes) {
        acc + bit_array.byte_size(bytes)
      })

    io.println("Frame with size: " <> int.to_string(frame_size))
  })
}
