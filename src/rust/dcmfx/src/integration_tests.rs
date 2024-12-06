// Integration tests for dcmfx
#[cfg(test)]
mod tests {
  const RNG_SEED: u64 = 1023;

  use std::{ffi::OsStr, fs::File, io::Read, io::Write, path::Path};

  use rand::rngs::SmallRng;
  use rand::{Rng, SeedableRng};
  use walkdir::WalkDir;

  use dcmfx_core::*;
  use dcmfx_json::*;
  use dcmfx_p10::*;

  // Integration tests are run with encapsulated pixel data allowed in the DICOM
  // JSON data
  const JSON_CONFIG: DicomJsonConfig = DicomJsonConfig {
    store_encapsulated_pixel_data: true,
  };

  #[test]
  fn integration_tests() -> Result<(), ()> {
    let test_assets_dir = if Path::new("../../test/assets").is_dir() {
      "../../test/assets"
    } else {
      "../../../test/assets"
    };

    // List all files in the test assets directory
    let data_files = WalkDir::new(test_assets_dir)
      .into_iter()
      .collect::<Result<Vec<_>, _>>()
      .unwrap();

    // Narrow down to just the DICOM files
    let mut dicoms = data_files
      .iter()
      .filter(|f| f.path().extension() == Some(OsStr::new("dcm")))
      .map(|f| f.path())
      .collect::<Vec<_>>();
    dicoms.sort();

    // Validate each file
    let validation_results: Vec<_> = dicoms
      .iter()
      .map(|dicom| validate_dicom(dicom).map_err(|e| (dicom, e)))
      .collect();

    // Print results
    if validation_results.iter().all(|r| r.is_ok()) {
      Ok(())
    } else {
      // Report details on failures
      for validation_result in validation_results {
        match validation_result {
          Ok(()) => (),

          Err((dicom, DicomValidationError::LoadError { error })) => {
            error.print(&format!("reading {:?}", dicom))
          }

          Err((dicom, DicomValidationError::PrintedOutputMissing)) => {
            eprintln!("Error: No printed output file for {:?}", dicom)
          }

          Err((dicom, DicomValidationError::PrintedOutputMismatch)) => {
            eprintln!(
              "Error: printed output mismatch with {:?}, compare the two files \
                for details",
              dicom
            )
          }

          Err((dicom, DicomValidationError::JsonOutputMissing)) => {
            eprintln!("Error: No JSON file for {:?}", dicom)
          }

          Err((dicom, DicomValidationError::JsonOutputMismatch)) => eprintln!(
            "Error: JSON mismatch with {:?}, compare the two files for details",
            dicom
          ),

          Err((dicom, DicomValidationError::RewriteMismatch)) => {
            eprintln!("Error: Rewrite of {:?} was different", dicom)
          }

          Err((dicom, DicomValidationError::JitteredReadError { error })) => {
            error.print(&format!("reading {:?} (jittered)", dicom));
          }

          Err((dicom, DicomValidationError::JitteredReadMismatch)) => {
            eprintln!("Error: Jittered read of {:?} was different", dicom);
          }
        }
      }

      Err(())
    }
  }

  enum DicomValidationError {
    LoadError { error: P10Error },
    PrintedOutputMissing,
    PrintedOutputMismatch,
    JsonOutputMissing,
    JsonOutputMismatch,
    RewriteMismatch,
    JitteredReadError { error: P10Error },
    JitteredReadMismatch,
  }

  /// Loads a DICOM file and checks that its JSON serialization by this library
  /// matches the expected JSON serialization stored alongside it on disk.
  ///
  fn validate_dicom(dicom: &Path) -> Result<(), DicomValidationError> {
    // Load the DICOM
    let data_set = DataSet::read_p10_file(dicom.to_str().unwrap())
      .map_err(|error| DicomValidationError::LoadError { error })?;

    // Read the expected JSON output from the associated .json file
    let expected_json_string =
      std::fs::read_to_string(format!("{}.json", dicom.to_string_lossy()))
        .map_err(|_| DicomValidationError::JsonOutputMissing)?;
    let expected_json: serde_json::Value =
      serde_json::from_str(&expected_json_string).unwrap();

    test_data_set_matches_expected_print_output(dicom, &data_set)?;
    test_data_set_matches_expected_json_output(
      dicom,
      &data_set,
      &expected_json,
    )?;
    test_dicom_json_rewrite_cycle(dicom, &expected_json_string)?;
    test_p10_rewrite_cycle(dicom, &data_set)?;

    // Test a read using a chunk size of 15 bytes (this isn't truly jittered as
    // the chunk size is constant)
    test_jittered_read(dicom, &data_set, &mut || 15)?;

    // Test a jittered read with chunk sizes ranging from 1 to 256 bytes
    let mut rng = SmallRng::seed_from_u64(RNG_SEED);
    test_jittered_read(dicom, &data_set, &mut || rng.gen_range(1..256))?;

    Ok(())
  }

  /// Tests that the printed output of the data is as expected.
  ///
  fn test_data_set_matches_expected_print_output(
    dicom: &Path,
    data_set: &DataSet,
  ) -> Result<(), DicomValidationError> {
    let expected_print_output =
      std::fs::read_to_string(format!("{}.printed", dicom.to_string_lossy()))
        .map_err(|_| DicomValidationError::PrintedOutputMissing)?;

    // Print the data set into a string
    let mut print_result = String::new();
    data_set.to_lines(
      &DataSetPrintOptions::new().styled(false).max_width(100),
      &mut |s| {
        print_result.push_str(&s);
        print_result.push('\n');
      },
    );

    // Compare the actual print output to the expected print output
    if print_result == *expected_print_output {
      Ok(())
    } else {
      // The printed output didn't match so write what was generated to a
      // separate file so it can be manually compared to find the discrepancy
      let mut file = File::create(format!(
        "{}.validation_failure.printed",
        dicom.to_string_lossy()
      ))
      .unwrap();

      file.write_all(print_result.as_bytes()).unwrap();
      file.flush().unwrap();

      Err(DicomValidationError::PrintedOutputMismatch)
    }
  }

  /// Tests that the JSON conversion of the data set matches the expected JSON
  /// content for the DICOM.
  ///
  fn test_data_set_matches_expected_json_output(
    dicom: &Path,
    data_set: &DataSet,
    expected_json: &serde_json::Value,
  ) -> Result<(), DicomValidationError> {
    // Convert the data set to JSON
    let data_set_json: serde_json::Value =
      serde_json::from_str(&data_set.to_json(JSON_CONFIG).unwrap()).unwrap();

    // Compare the actual JSON to the expected JSON
    if data_set_json == *expected_json {
      Ok(())
    } else {
      // The JSON didn't match so write what was generated to a separate JSON
      // file so it can be manually compared to find the discrepancy
      let data_set_json = data_set_json.to_string();
      let mut file = File::create(format!(
        "{}.validation_failure.json",
        dicom.to_string_lossy()
      ))
      .unwrap();

      file.write_all(data_set_json.as_bytes()).unwrap();
      file.flush().unwrap();

      Err(DicomValidationError::JsonOutputMismatch)
    }
  }

  /// Tests that the conversion of the given DICOM JSON content is unchanged
  /// when converted to a data set and then converted back to DICOM JSON.
  ///
  fn test_dicom_json_rewrite_cycle(
    dicom: &Path,
    expected_json_string: &str,
  ) -> Result<(), DicomValidationError> {
    let original_json: serde_json::Value =
      serde_json::from_str(expected_json_string).unwrap();

    // Check the reverse by converting the expected JSON to a data set then back
    // to JSON and checking it matches the original. This tests the reading of
    // DICOM JSON data into a data set.
    let data_set = DataSet::from_json(expected_json_string).unwrap();
    let data_set_json: serde_json::Value =
      serde_json::from_str(&data_set.to_json(JSON_CONFIG).unwrap()).unwrap();

    // Compare the actual JSON to the expected JSON
    if original_json == data_set_json {
      Ok(())
    } else {
      // The JSON didn't match so write what was generated to a separate JSON
      // file so it can be manually compared to find the discrepancy

      let mut file = File::create(format!(
        "{}.validation_failure.json",
        dicom.to_string_lossy()
      ))
      .unwrap();

      file
        .write_all(data_set_json.to_string().as_bytes())
        .unwrap();
      file.flush().unwrap();

      Err(DicomValidationError::JsonOutputMismatch)
    }
  }

  /// Puts a data set through a full write and read cycle and checks that
  /// nothing changes.
  ///
  fn test_p10_rewrite_cycle(
    dicom: &Path,
    data_set: &DataSet,
  ) -> Result<(), DicomValidationError> {
    let tmp_file = format!("{}.tmp", dicom.to_string_lossy());
    data_set.write_p10_file(&tmp_file, None).unwrap();
    let rewritten_data_set = DataSet::read_p10_file(&tmp_file).unwrap();
    std::fs::remove_file(tmp_file).unwrap();

    // Filter that removes File Meta Information and specific character set data
    // elements which we don't want to be part of the rewrite comparison
    let data_set_filter =
      |(tag, _value): &(&DataElementTag, &DataElementValue)| {
        tag.group != 0x0002 && **tag != dictionary::SPECIFIC_CHARACTER_SET.tag
      };

    let data_set: DataSet = data_set
      .iter()
      .filter(data_set_filter)
      .map(|(tag, value)| (*tag, value.clone()))
      .collect();

    let rewritten_data_set: DataSet = rewritten_data_set
      .iter()
      .filter(data_set_filter)
      .map(|(tag, value)| (*tag, value.clone()))
      .collect();

    if data_set == rewritten_data_set {
      Ok(())
    } else {
      Err(DicomValidationError::RewriteMismatch)
    }
  }

  /// Reads a DICOM in streaming fashion with each chunk of incoming P10 data
  /// being of a random size. This tests that DICOM reading is unaffected by
  /// different input chunk sizes and where the boundaries between chunks fall.
  ///
  fn test_jittered_read(
    dicom: &Path,
    data_set: &DataSet,
    next_chunk_size: &mut impl FnMut() -> usize,
  ) -> Result<(), DicomValidationError> {
    let mut file = File::open(dicom).unwrap();

    let mut context = P10ReadContext::new();
    let mut data_set_builder = DataSetBuilder::new();

    while !data_set_builder.is_complete() {
      match context.read_parts() {
        Ok(parts) => {
          for part in parts {
            data_set_builder.add_part(&part).unwrap()
          }
        }

        Err(P10Error::DataRequired { .. }) => {
          let mut buffer = vec![0u8; next_chunk_size()];

          match file.read(&mut buffer).unwrap() {
            0 => context.write_bytes(vec![], true).unwrap(),

            bytes_count => {
              buffer.resize(bytes_count, 0);
              context.write_bytes(buffer, false).unwrap();
            }
          }
        }

        Err(error) => {
          return Err(DicomValidationError::JitteredReadError { error })
        }
      }
    }

    if *data_set != data_set_builder.final_data_set().unwrap() {
      return Err(DicomValidationError::JitteredReadMismatch);
    }

    Ok(())
  }
}
