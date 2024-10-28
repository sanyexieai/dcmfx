## Importing

:::tabs key:code-example

== Gleam
To use DCMfx from Gleam, add each library then import its required modules.

```gleam
// Add packages: `gleam add dcmfx_core dcmfx_p10`

import dcmfx_core/data_set
import dcmfx_p10

pub fn main() {
  let assert Ok(ds) = dcmfx_p10.read_file("input.dcm")
  data_set.print(ds, None)
}
```

== Rust
To use DCMfx from Rust, add the `dcmfx` crate then access each library
via its namespace.

```rust
// Add crate: `cargo add dcmfx`

use dcmfx::core::*;
use dcmfx::p10::*;

pub fn main() {
  let ds: DataSet = DataSet::read_p10_file("input.dcm").unwrap();
  ds.print(None);
}
```
:::
