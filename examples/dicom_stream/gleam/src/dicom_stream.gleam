import dcmfx_p10
import dcmfx_p10/p10_error.{type P10Error}
import dcmfx_p10/p10_read.{type P10ReadContext}
import dcmfx_p10/p10_write.{type P10WriteContext}
import file_streams/file_stream.{type FileStream}
import gleam/result

const input_file = "../../example.dcm"

const output_file = "output.dcm"

pub fn main() -> Result(Nil, P10Error) {
  let assert Ok(input_stream) = file_stream.open_read(input_file)
  let assert Ok(output_stream) = file_stream.open_write(output_file)

  let read_context = p10_read.new_read_context()
  let write_context = p10_write.new_write_context()

  do_stream(input_stream, output_stream, read_context, write_context)
}

fn do_stream(
  input_stream: FileStream,
  output_stream: FileStream,
  read_context: P10ReadContext,
  write_context: P10WriteContext,
) -> Result(Nil, P10Error) {
  use #(parts, read_context) <- result.try(dcmfx_p10.read_parts_from_stream(
    input_stream,
    read_context,
  ))

  use #(ended, write_context) <- result.try(dcmfx_p10.write_parts_to_stream(
    parts,
    output_stream,
    write_context,
  ))

  case ended {
    True -> Ok(Nil)
    False -> do_stream(input_stream, output_stream, read_context, write_context)
  }
}
