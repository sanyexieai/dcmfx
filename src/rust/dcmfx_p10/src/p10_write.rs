//! Functionality for serializing data sets and streams of DICOM P10 parts into
//! DICOM P10 bytes.

use std::rc::Rc;

use byteorder::ByteOrder;

use dcmfx_core::DataSetPath;
use dcmfx_core::{
  dictionary, transfer_syntax, transfer_syntax::Endianness, DataElementTag,
  DataElementValue, DataSet, TransferSyntax,
};

use crate::{
  internal::{
    data_element_header::{DataElementHeader, ValueLengthSize},
    value_length::ValueLength,
  },
  p10_part, uids, P10Error, P10FilterTransform, P10InsertTransform, P10Part,
};

/// Data is compressed into chunks of this size when writing deflated transfer
/// syntaxes.
///
const ZLIB_DEFLATE_CHUNK_SIZE: usize = 64 * 1024;

/// Configuration used when writing DICOM P10 data.
///
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct P10WriteConfig {
  /// The zlib compression level to use when the transfer syntax being used is
  /// deflated. There are only three deflated transfer syntaxes: 'Deflated
  /// Explicit VR Little Endian', 'JPIP Referenced Deflate', and 'JPIP HTJ2K
  /// Referenced Deflate'.
  ///
  /// The level ranges from 0, meaning no compression, through to 9, which gives
  /// the best compression at the cost of speed.
  ///
  /// Default: 6.
  pub zlib_compression_level: u32,
}

impl Default for P10WriteConfig {
  fn default() -> Self {
    Self {
      zlib_compression_level: 6,
    }
  }
}

/// A write context holds the current state of an in-progress DICOM P10 write.
/// DICOM P10 parts are written to a write context with [`Self::write_part()`],
/// and output P10 bytes are returned by [`Self::read_bytes()`].
///
pub struct P10WriteContext {
  config: P10WriteConfig,
  p10_bytes: Vec<Rc<Vec<u8>>>,
  p10_total_byte_count: u64,
  is_ended: bool,
  transfer_syntax: &'static TransferSyntax,
  zlib_stream: Option<flate2::Compress>,
  path: DataSetPath,
  sequence_item_counts: Vec<usize>,
}

impl P10WriteContext {
  /// Creates a new write context for writing DICOM P10 data.
  ///
  pub fn new() -> Self {
    Self {
      config: P10WriteConfig::default(),
      p10_bytes: vec![],
      p10_total_byte_count: 0,
      is_ended: false,
      transfer_syntax: &transfer_syntax::IMPLICIT_VR_LITTLE_ENDIAN,
      zlib_stream: None,
      path: DataSetPath::new(),
      sequence_item_counts: vec![],
    }
  }

  /// Updates the config for a write context.
  ///
  pub fn set_config(&mut self, config: &P10WriteConfig) {
    // Clamp zlib compression level to the valid range
    self.config.zlib_compression_level =
      config.zlib_compression_level.clamp(0, 9);
  }

  /// Reads the current DICOM P10 bytes available out of a write context. These
  /// are the bytes generated by recent calls to [`Self::write_part()`].
  ///
  pub fn read_bytes(&mut self) -> Vec<Rc<Vec<u8>>> {
    std::mem::take(&mut self.p10_bytes)
  }

  /// Writes a DICOM P10 part to a write context. On success an updated write
  /// context is returned. Use [`Self::read_bytes()`] to get the new DICOM P10
  /// bytes generated as a result of writing this part.
  ///
  pub fn write_part(&mut self, part: &P10Part) -> Result<(), P10Error> {
    if self.is_ended {
      return Err(P10Error::PartStreamInvalid {
        when: "Writing DICOM P10 part".to_string(),
        details:
          "Received a further DICOM P10 part after the write was completed"
            .to_string(),
        part: part.clone(),
      });
    }

    match part {
      // When the File Meta Information part is received, check it for a
      // transfer syntax value that should be put onto the write context, and
      // start a zlib compressor if the transfer syntax is deflated
      P10Part::FileMetaInformation {
        data_set: ref file_meta_information,
      } => {
        // Read the transfer syntax UID
        let transfer_syntax_uid = file_meta_information
          .get_string(dictionary::TRANSFER_SYNTAX_UID.tag)
          .unwrap_or(transfer_syntax::IMPLICIT_VR_LITTLE_ENDIAN.uid);

        // Map UID to a known transfer syntax
        let new_transfer_syntax = TransferSyntax::from_uid(transfer_syntax_uid)
          .map_err(|_| P10Error::TransferSyntaxNotSupported {
            transfer_syntax_uid: transfer_syntax_uid.to_string(),
          })?;

        // If this is a deflated transfer syntax then start a zlib compressor
        // and exclude the zlib header
        if new_transfer_syntax.is_deflated {
          self.zlib_stream = Some(flate2::Compress::new(
            flate2::Compression::new(self.config.zlib_compression_level),
            false,
          ));
        }

        self.transfer_syntax = new_transfer_syntax;

        let part_bytes = self.part_to_bytes(part)?;
        self.p10_total_byte_count += part_bytes.len() as u64;
        self.p10_bytes.push(part_bytes);

        Ok(())
      }

      // When the end part is received, update the flag on the write context and
      // flush all remaining data out of the zlib stream if one is in use
      P10Part::End => {
        if let Some(zlib_stream) = self.zlib_stream.as_mut() {
          loop {
            let mut output = vec![0u8; ZLIB_DEFLATE_CHUNK_SIZE];

            let total_out = zlib_stream.total_out();
            let status = zlib_stream
              .compress(
                &[],
                output.as_mut_slice(),
                flate2::FlushCompress::Finish,
              )
              .unwrap();
            output.resize((zlib_stream.total_out() - total_out) as usize, 0u8);

            if !output.is_empty() {
              self.p10_total_byte_count += output.len() as u64;
              self.p10_bytes.push(Rc::new(output));
            }

            if status == flate2::Status::StreamEnd {
              break;
            }
          }

          self.zlib_stream = None;
        }

        self.is_ended = true;

        Ok(())
      }

      _ => {
        // Update the current path
        match part {
          P10Part::DataElementHeader { tag, .. } => {
            self.path.add_data_element(*tag)
          }

          P10Part::SequenceStart { tag, .. } => {
            self.sequence_item_counts.push(0);
            self.path.add_data_element(*tag)
          }

          P10Part::SequenceItemStart | P10Part::PixelDataItem { .. } => {
            let index = self.sequence_item_counts.last_mut().unwrap();

            *index += 1;
            self.path.add_sequence_item(*index - 1)
          }

          _ => Ok(()),
        }
        .map_err(|_| P10Error::PartStreamInvalid {
          when: "Writing part to context".to_string(),
          details: "The data set path is not in a valid state for this part"
            .to_string(),
          part: part.clone(),
        })?;

        // Convert part to bytes
        let part_bytes = self.part_to_bytes(part)?;

        // Update the current path
        match part {
          P10Part::DataElementValueBytes {
            bytes_remaining: 0, ..
          }
          | P10Part::SequenceItemDelimiter => self.path.pop(),

          P10Part::SequenceDelimiter => {
            self.sequence_item_counts.pop();
            self.path.pop()
          }

          _ => Ok(()),
        }
        .map_err(|_| P10Error::PartStreamInvalid {
          when: "Writing part to context".to_string(),
          details: "The data set path is empty".to_string(),
          part: part.clone(),
        })?;

        // If a zlib stream is active then pass the P10 bytes through it
        if let Some(zlib_stream) = self.zlib_stream.as_mut() {
          let mut part_bytes_remaining = &part_bytes[..];

          while !part_bytes_remaining.is_empty() {
            let mut output = vec![0u8; ZLIB_DEFLATE_CHUNK_SIZE];

            // Add bytes to the zlib compressor and read back any compressed
            // data
            let total_in = zlib_stream.total_in();
            let total_out = zlib_stream.total_out();
            zlib_stream
              .compress(
                part_bytes_remaining,
                &mut output,
                flate2::FlushCompress::None,
              )
              .unwrap();
            output.resize((zlib_stream.total_out() - total_out) as usize, 0u8);

            if !output.is_empty() {
              self.p10_total_byte_count += output.len() as u64;
              self.p10_bytes.push(Rc::new(output));
            }

            let input_bytes_consumed =
              (zlib_stream.total_in() - total_in) as usize;
            if input_bytes_consumed == 0 {
              panic!("zlib compressor did not consume any bytes");
            }

            part_bytes_remaining =
              &part_bytes_remaining[input_bytes_consumed..];
          }
        } else {
          self.p10_total_byte_count += part_bytes.len() as u64;
          self.p10_bytes.push(part_bytes);
        }

        Ok(())
      }
    }
  }

  /// Converts a single DICOM P10 part to raw DICOM P10 bytes.
  ///
  fn part_to_bytes(&self, part: &P10Part) -> Result<Rc<Vec<u8>>, P10Error> {
    match part {
      P10Part::FilePreambleAndDICMPrefix { preamble } => {
        let mut data = Vec::with_capacity(132);

        data.extend_from_slice(preamble.as_ref());
        data.extend_from_slice(b"DICM");

        Ok(Rc::new(data))
      }

      P10Part::FileMetaInformation { data_set } => {
        let mut file_meta_information = data_set.clone();
        prepare_file_meta_information_part_data_set(&mut file_meta_information);

        let mut fmi_bytes = Vec::with_capacity(8192);

        // Set the File Meta Information Group Length, with a placeholder for the
        // 32-bit length at the end. The length will be filled in once the rest of
        // the FMI bytes have been created.
        fmi_bytes
          .extend_from_slice(&[0x02, 0x00, 0x00, 0x00, 0x55, 0x4C, 0x04, 0x00]);
        fmi_bytes.extend_from_slice(&[0, 0, 0, 0]);

        for (tag, value) in file_meta_information.into_iter() {
          let vr = value.value_representation();

          let value_bytes =
            value.bytes().map_err(|_| P10Error::DataInvalid {
              when: "Serializing File Meta Information".to_string(),
              details: format!(
            "Tag '{}' with value representation '{}' is not allowed in File \
              Meta Information",
            tag, vr
          ),
              path: self.path.clone(),
              offset: self.p10_total_byte_count,
            })?;

          let header_bytes = self.data_element_header_to_bytes(
            &DataElementHeader {
              tag,
              vr: Some(vr),
              length: ValueLength::new(value_bytes.len() as u32),
            },
            transfer_syntax::Endianness::LittleEndian,
          )?;

          fmi_bytes.extend_from_slice(&header_bytes);
          fmi_bytes.extend_from_slice(value_bytes);
        }

        // Set the final File Meta Information Group Length value
        let fmi_length = fmi_bytes.len() - 12;
        byteorder::LittleEndian::write_u32_into(
          &[fmi_length as u32],
          &mut fmi_bytes[8..12],
        );

        Ok(Rc::new(fmi_bytes))
      }

      P10Part::DataElementHeader { tag, vr, length } => {
        let vr = match self.transfer_syntax.vr_serialization {
          transfer_syntax::VrSerialization::VrExplicit => Some(*vr),
          transfer_syntax::VrSerialization::VrImplicit => None,
        };

        self.data_element_header_to_bytes(
          &DataElementHeader {
            tag: *tag,
            vr,
            length: ValueLength::new(*length),
          },
          self.transfer_syntax.endianness,
        )
      }

      P10Part::DataElementValueBytes { vr, data, .. } => {
        if self.transfer_syntax.endianness.is_big() {
          // To swap endianness the data needs to be cloned as it can't be swapped
          // in place
          let mut data_vec = (**data).clone();
          vr.swap_endianness(&mut data_vec);
          Ok(Rc::new(data_vec))
        } else {
          Ok(data.clone())
        }
      }

      P10Part::SequenceStart { tag, vr } => {
        let vr = match self.transfer_syntax.vr_serialization {
          transfer_syntax::VrSerialization::VrExplicit => Some(*vr),
          transfer_syntax::VrSerialization::VrImplicit => None,
        };

        self.data_element_header_to_bytes(
          &DataElementHeader {
            tag: *tag,
            vr,
            length: ValueLength::Undefined,
          },
          self.transfer_syntax.endianness,
        )
      }

      P10Part::SequenceDelimiter => self.data_element_header_to_bytes(
        &DataElementHeader {
          tag: dictionary::SEQUENCE_DELIMITATION_ITEM.tag,
          vr: None,
          length: ValueLength::ZERO,
        },
        self.transfer_syntax.endianness,
      ),

      P10Part::SequenceItemStart => self.data_element_header_to_bytes(
        &DataElementHeader {
          tag: dictionary::ITEM.tag,
          vr: None,
          length: ValueLength::Undefined,
        },
        self.transfer_syntax.endianness,
      ),

      P10Part::SequenceItemDelimiter => self.data_element_header_to_bytes(
        &DataElementHeader {
          tag: dictionary::ITEM_DELIMITATION_ITEM.tag,
          vr: None,
          length: ValueLength::ZERO,
        },
        self.transfer_syntax.endianness,
      ),

      P10Part::PixelDataItem { length } => self.data_element_header_to_bytes(
        &DataElementHeader {
          tag: dictionary::ITEM.tag,
          vr: None,
          length: ValueLength::new(*length),
        },
        self.transfer_syntax.endianness,
      ),

      P10Part::End => Ok(Rc::new(vec![])),
    }
  }

  /// Serializes a data element header to a `Vec<u8>`. If the VR is not
  /// specified then the transfer syntax is assumed to use implicit VRs.
  ///
  fn data_element_header_to_bytes(
    &self,
    header: &DataElementHeader,
    endianness: Endianness,
  ) -> Result<Rc<Vec<u8>>, P10Error> {
    let length = header.length.to_u32();

    let mut bytes = Vec::with_capacity(12);

    match endianness {
      Endianness::LittleEndian => {
        bytes.extend_from_slice(header.tag.group.to_le_bytes().as_slice());
        bytes.extend_from_slice(header.tag.element.to_le_bytes().as_slice());
      }
      Endianness::BigEndian => {
        bytes.extend_from_slice(header.tag.group.to_be_bytes().as_slice());
        bytes.extend_from_slice(header.tag.element.to_be_bytes().as_slice());
      }
    };

    match header.vr {
      // Write with implicit VR
      None => match endianness {
        Endianness::LittleEndian => {
          bytes.extend_from_slice(length.to_le_bytes().as_slice())
        }
        Endianness::BigEndian => {
          bytes.extend_from_slice(length.to_be_bytes().as_slice())
        }
      },

      // Write with explicit VR
      Some(vr) => {
        bytes.extend_from_slice(vr.to_string().as_bytes());

        match DataElementHeader::value_length_size(vr) {
          // All other VRs use a 16-bit length. Check that the data length fits
          // inside this constraint.
          ValueLengthSize::U16 => {
            if length > u16::MAX as u32 {
              return Err(P10Error::DataInvalid {
                when: "Serializing data element header".to_string(),
                details: format!(
                  "Length 0x{:X} exceeds the maximum of 0xFFFF",
                  header.length.to_u32(),
                ),
                path: self.path.clone(),
                offset: self.p10_total_byte_count,
              });
            }

            match endianness {
              Endianness::LittleEndian => bytes
                .extend_from_slice((length as u16).to_le_bytes().as_slice()),
              Endianness::BigEndian => bytes
                .extend_from_slice((length as u16).to_be_bytes().as_slice()),
            }
          }

          // The following VRs use a 32-bit length preceded by two padding bytes
          ValueLengthSize::U32 => {
            bytes.extend_from_slice([0, 0].as_slice());

            match endianness {
              Endianness::LittleEndian => {
                bytes.extend_from_slice(length.to_le_bytes().as_slice())
              }
              Endianness::BigEndian => {
                bytes.extend_from_slice(length.to_be_bytes().as_slice())
              }
            }
          }
        };
      }
    }

    Ok(Rc::new(bytes))
  }
}

impl Default for P10WriteContext {
  fn default() -> Self {
    Self::new()
  }
}

/// Converts a data set to DICOM P10 parts. The generated P10 parts are returned
/// via a callback.
///
pub fn data_set_to_parts<E>(
  data_set: &DataSet,
  part_callback: &mut impl FnMut(&P10Part) -> Result<(), E>,
) -> Result<(), E> {
  // Create filter transform that removes File Meta Information data elements
  // from the data set's part stream
  let mut remove_fmi_transform = P10FilterTransform::new(
    Box::new(|tag: DataElementTag, _, _| tag.group != 2),
    false,
  );

  // Create insert transform to add the '(0008,0005) SpecificCharacterSet' data
  // element into the data set's part stream, specifying UTF-8 (ISO_IR 192)
  let mut data_elements_to_insert = DataSet::new();
  data_elements_to_insert
    .insert_string_value(&dictionary::SPECIFIC_CHARACTER_SET, &["ISO_IR 192"])
    .unwrap();
  let mut insert_specific_character_set_transform =
    P10InsertTransform::new(data_elements_to_insert);

  // Create a function that passes parts through the above two transforms and
  // then to the callback
  let mut process_part = |part: &P10Part| -> Result<(), E> {
    if !remove_fmi_transform.add_part(part) {
      return Ok(());
    }

    let parts = insert_specific_character_set_transform.add_part(part);

    for part in parts {
      part_callback(&part)?;
    }

    Ok(())
  };

  // Write File Preamble and File Meta Information parts
  let preamble_part = P10Part::FilePreambleAndDICMPrefix {
    preamble: Box::new([0; 128]),
  };
  process_part(&preamble_part)?;
  let fmi_part = P10Part::FileMetaInformation {
    data_set: data_set.file_meta_information(),
  };
  process_part(&fmi_part)?;

  // Write main data set
  p10_part::data_elements_to_parts(data_set, &mut process_part)?;

  // Write end part
  process_part(&P10Part::End)
}

/// Converts a data set to DICOM P10 bytes. The generated P10 bytes are returned
/// via a callback.
///
pub fn data_set_to_bytes(
  data_set: &DataSet,
  bytes_callback: &mut impl FnMut(Rc<Vec<u8>>) -> Result<(), P10Error>,
  config: &P10WriteConfig,
) -> Result<(), P10Error> {
  let mut context = P10WriteContext::new();
  context.set_config(config);

  let mut process_part = |part: &P10Part| -> Result<(), P10Error> {
    context.write_part(part)?;

    let p10_bytes = context.read_bytes();
    for bytes in p10_bytes {
      bytes_callback(bytes)?;
    }

    Ok(())
  };

  data_set_to_parts(data_set, &mut process_part)
}

/// Sets the *'(0002,0001) File Meta Information Version'*, *'(0002,0012)
/// Implementation Class UID'* and *'(0002,0013) Implementation Version Name'*
/// values in the File Meta Information. This is done prior to serializing it
/// to bytes.
///
fn prepare_file_meta_information_part_data_set(
  file_meta_information: &mut DataSet,
) {
  let file_meta_information_version =
    DataElementValue::new_other_byte_string(vec![0, 1]).unwrap();
  let implementation_class_uid = DataElementValue::new_unique_identifier(&[
    uids::DCMFX_IMPLEMENTATION_CLASS_UID,
  ])
  .unwrap();

  let implementation_version_name = DataElementValue::new_short_string(&[
    &uids::DCMFX_IMPLEMENTATION_VERSION_NAME,
  ])
  .unwrap();

  file_meta_information.insert(
    dictionary::FILE_META_INFORMATION_VERSION.tag,
    file_meta_information_version,
  );
  file_meta_information.insert(
    dictionary::IMPLEMENTATION_CLASS_UID.tag,
    implementation_class_uid,
  );
  file_meta_information.insert(
    dictionary::IMPLEMENTATION_VERSION_NAME.tag,
    implementation_version_name,
  )
}

#[cfg(test)]
mod tests {
  use super::*;

  use dcmfx_core::ValueRepresentation;

  #[test]
  fn data_element_header_to_bytes_test() {
    assert_eq!(
      P10WriteContext::new().data_element_header_to_bytes(
        &DataElementHeader {
          tag: dictionary::WAVEFORM_DATA.tag,
          vr: None,
          length: ValueLength::new(0x12345678),
        },
        Endianness::LittleEndian,
      ),
      Ok(Rc::new(vec![0, 84, 16, 16, 120, 86, 52, 18]))
    );

    assert_eq!(
      P10WriteContext::new().data_element_header_to_bytes(
        &DataElementHeader {
          tag: dictionary::WAVEFORM_DATA.tag,
          vr: None,
          length: ValueLength::new(0x12345678),
        },
        Endianness::BigEndian,
      ),
      Ok(Rc::new(vec![84, 0, 16, 16, 18, 52, 86, 120]))
    );

    assert_eq!(
      P10WriteContext::new().data_element_header_to_bytes(
        &DataElementHeader {
          tag: dictionary::PATIENT_AGE.tag,
          vr: Some(ValueRepresentation::UnlimitedText),
          length: ValueLength::new(0x1234),
        },
        Endianness::LittleEndian,
      ),
      Ok(Rc::new(vec![16, 0, 16, 16, 85, 84, 0, 0, 52, 18, 0, 0]))
    );

    assert_eq!(
      P10WriteContext::new().data_element_header_to_bytes(
        &DataElementHeader {
          tag: dictionary::PIXEL_DATA.tag,
          vr: Some(ValueRepresentation::OtherWordString),
          length: ValueLength::new(0x12345678),
        },
        Endianness::LittleEndian,
      ),
      Ok(Rc::new(vec![
        224, 127, 16, 0, 79, 87, 0, 0, 120, 86, 52, 18
      ]))
    );

    assert_eq!(
      P10WriteContext::new().data_element_header_to_bytes(
        &DataElementHeader {
          tag: dictionary::PIXEL_DATA.tag,
          vr: Some(ValueRepresentation::OtherWordString),
          length: ValueLength::new(0x12345678),
        },
        Endianness::BigEndian,
      ),
      Ok(Rc::new(vec![
        127, 224, 0, 16, 79, 87, 0, 0, 18, 52, 86, 120
      ]))
    );

    assert_eq!(
      P10WriteContext::new().data_element_header_to_bytes(
        &DataElementHeader {
          tag: dictionary::PATIENT_AGE.tag,
          vr: Some(ValueRepresentation::AgeString),
          length: ValueLength::new(0x12345),
        },
        Endianness::LittleEndian,
      ),
      Err(P10Error::DataInvalid {
        when: "Serializing data element header".to_string(),
        details: "Length 0x12345 exceeds the maximum of 0xFFFF".to_string(),
        path: DataSetPath::new(),
        offset: 0
      })
    );

    assert_eq!(
      P10WriteContext::new().data_element_header_to_bytes(
        &DataElementHeader {
          tag: dictionary::SMALLEST_IMAGE_PIXEL_VALUE.tag,
          vr: Some(ValueRepresentation::SignedShort),
          length: ValueLength::new(0x1234),
        },
        Endianness::LittleEndian,
      ),
      Ok(Rc::new(vec![40, 0, 6, 1, 83, 83, 52, 18]))
    );

    assert_eq!(
      P10WriteContext::new().data_element_header_to_bytes(
        &DataElementHeader {
          tag: dictionary::SMALLEST_IMAGE_PIXEL_VALUE.tag,
          vr: Some(ValueRepresentation::SignedShort),
          length: ValueLength::new(0x1234),
        },
        Endianness::BigEndian,
      ),
      Ok(Rc::new(vec![0, 40, 1, 6, 83, 83, 18, 52]))
    );
  }
}
