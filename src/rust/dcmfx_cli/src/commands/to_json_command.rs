use std::fs::File;
use std::io::{Read, Write};

use clap::Args;

use dcmfx::core::*;
use dcmfx::json::*;
use dcmfx::p10::*;

pub const ABOUT: &str = "Converts a DICOM P10 file to a DICOM JSON file";

#[derive(Args)]
pub struct ToJsonArgs {
  #[clap(
    help = "The name of the file to read DICOM P10 content from. Specify '-' \
      to read from stdin."
  )]
  input_filename: String,

  #[clap(
    help = "The name of the file to write DICOM JSON content to. Specify '-' \
      to write to stdout."
  )]
  output_filename: String,

  #[arg(
    long,
    short = 'p',
    help = "Whether to extend the DICOM JSON model to store encapsulated pixel \
      data as inline binaries",
    default_value_t = false
  )]
  store_encapsulated_pixel_data: bool,
}

pub fn run(args: &ToJsonArgs) -> Result<(), ()> {
  let config = DicomJsonConfig {
    store_encapsulated_pixel_data: args.store_encapsulated_pixel_data,
  };

  match perform_to_json(&args.input_filename, &args.output_filename, &config) {
    Ok(()) => Ok(()),
    Err(e) => {
      e.print(&format!("converting \"{}\" to JSON", args.input_filename));
      Err(())
    }
  }
}

fn perform_to_json(
  input_filename: &str,
  output_filename: &str,
  config: &DicomJsonConfig,
) -> Result<(), Box<dyn DcmfxError>> {
  // Open input stream
  let mut input_stream: Box<dyn Read> = match input_filename {
    "-" => Box::new(std::io::stdin()),
    _ => match File::open(input_filename) {
      Ok(file) => Box::new(file),
      Err(e) => {
        return Err(Box::new(P10Error::FileError {
          when: "Opening input file".to_string(),
          details: e.to_string(),
        }));
      }
    },
  };

  // Open output stream
  let mut output_stream: Box<dyn Write> = match output_filename {
    "-" => Box::new(std::io::stdout()),
    _ => match File::create(output_filename) {
      Ok(file) => Box::new(file),
      Err(e) => {
        return Err(Box::new(P10Error::FileError {
          when: "Opening output file".to_string(),
          details: e.to_string(),
        }));
      }
    },
  };

  // Create P10 read context and set max part size to 256 KiB
  let mut context = P10ReadContext::new();
  context.set_config(&P10ReadConfig {
    max_part_size: 256 * 1024,
    ..P10ReadConfig::default()
  });

  // Create transform for converting P10 parts into bytes of JSON
  let mut json_transform = P10JsonTransform::new(config);

  loop {
    // Read the next parts from the input
    let parts =
      match dcmfx::p10::read_parts_from_stream(&mut input_stream, &mut context)
      {
        Ok(parts) => parts,
        Err(e) => return Err(Box::new(e)),
      };

    // Write the parts to the JSON transform, directing the resulting JSON to
    // the output stream
    for part in parts.iter() {
      match json_transform.add_part(part, &mut output_stream) {
        Ok(()) => (),
        Err(JsonSerializeError::DataError(e)) => return Err(Box::new(e)),
        Err(JsonSerializeError::IOError(e)) => {
          return Err(Box::new(P10Error::FileError {
            when: "Writing output file".to_string(),
            details: e.to_string(),
          }));
        }
      };

      // When the end part has been written the conversion is complete
      if *part == P10Part::End {
        return match output_stream.flush() {
          Ok(()) => Ok(()),
          Err(e) => Err(Box::new(P10Error::FileError {
            when: "Writing output file".to_string(),
            details: e.to_string(),
          })),
        };
      }
    }
  }
}
