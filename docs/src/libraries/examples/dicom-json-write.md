# Example: Write DICOM JSON

The following code reads a DICOM P10 file, converts it to DICOM JSON, then
prints the JSON to stdout.

Note that the extension to DICOM JSON allowing storage of encapsulated pixel
data is enabled. If this is omitted then conversion of a data set containing
encapsulated pixel data will error.

:::tabs key:code-example
== Gleam
<<< @/../../examples/dicom_json_write/gleam/src/dicom_json_write.gleam
== Rust
<<< @/../../examples/dicom_json_write/rust/src/main.rs
:::

[View on GitHub](https://github.com/dcmfx/dcmfx/tree/main/examples/dicom_json_write)
