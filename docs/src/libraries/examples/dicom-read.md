# Example: Read DICOM File

The following code reads a DICOM P10 file and prints its full contents to
stdout, then extracts the patient ID and study date data elements and prints
them individually.

:::tabs key:code-example
== Gleam
<<< @/../../examples/dicom_read/gleam/src/dicom_read.gleam
== Rust
<<< @/../../examples/dicom_read/rust/src/main.rs
:::

[View on GitHub](https://github.com/dcmfx/dcmfx/tree/main/examples/dicom_read)
