# Design

DCMfx's design is focused around streaming DICOM data, i.e. all operations are
performed in a streaming fashion wherever possible, enabling fast operation
while with extremely low memory usage regardless of DICOM or data set size.

Loading DICOM data sets completely into memory is also supported, if preferred.

## Languages

DCMfx is dual-implemented in two languages: [Gleam](https://gleam.run) and
[Rust](https://rust-lang.org). See [here](./libraries/overview#languages) for
more details.
