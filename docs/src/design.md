# Design

DCMfx is built with a streaming-centered design, which means that all operations
operate in a streaming fashion wherever possible, enabling fast operation while
keeping memory usage extremely low regardless of DICOM or data set size.

Materializing data sets in memory is also supported, and is built on top of the
underlying stream-based design.

## Languages

DCMfx is dual-implemented in two languages: [Gleam](https://gleam.run) and
[Rust](https://rust-lang.org). See [here](./libraries/overview#languages) for
more details.
