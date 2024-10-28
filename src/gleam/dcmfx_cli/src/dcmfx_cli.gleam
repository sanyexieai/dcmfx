import argv
import birl
import dcmfx_cli/commands/extract_pixel_data_command
import dcmfx_cli/commands/modify_command
import dcmfx_cli/commands/print_command
import dcmfx_cli/commands/to_dcm_command
import dcmfx_cli/commands/to_json_command
import gleam/float
import gleam/int
import gleam/io
import gleam/list
import glint

fn print_stats_flag() {
  glint.bool_flag("print-stats")
  |> glint.flag_default(False)
  |> glint.flag_help("Write timing and memory stats to stderr on exit")
}

pub fn main() {
  let args = argv.load().arguments

  let started_at = birl.monotonic_now()

  glint.new()
  |> glint.with_name("dcmfx")
  |> glint.global_help(
    "DCMfx is a CLI app for working with DICOM and DICOM JSON",
  )
  |> glint.with_max_output_width(80)
  |> glint.pretty_help(glint.default_pretty_help())
  |> glint.group_flag([], print_stats_flag())
  |> glint.add(["print"], print_command.run())
  |> glint.add(["to-dcm"], to_dcm_command.run())
  |> glint.add(["to-json"], to_json_command.run())
  |> glint.add(["extract-pixel-data"], extract_pixel_data_command.run())
  |> glint.add(["modify"], modify_command.run())
  |> glint.run_and_handle(args, fn(command_result) {
    case list.contains(args, "--print-stats") {
      True -> {
        let elapsed_ms = { birl.monotonic_now() - started_at } / 1000
        let elapsed_seconds = int.to_float(elapsed_ms) /. 1000.0

        io.println_error("")
        io.println_error("-----")
        io.println_error(
          "Time elapsed:       "
          <> float.to_string(elapsed_seconds)
          <> " seconds",
        )

        let total_memory_usage =
          { int.to_float(memory(Total)) /. 1024.0 /. 1024.0 }
          |> float.truncate
          |> int.to_string

        io.println_error("Total memory usage: " <> total_memory_usage <> " MiB")
      }
      False -> Nil
    }

    case command_result {
      Ok(Nil) -> Nil
      Error(Nil) -> exit_with_status(1)
    }
  })
}

@external(erlang, "erlang", "halt")
fn exit_with_status(status: Int) -> Nil

// TODO: determine if the `maximum` value mentioned in the docs can be made to
// work
type MemoryType {
  Total
}

@external(erlang, "erlang", "memory")
fn memory(memory_type: MemoryType) -> Int