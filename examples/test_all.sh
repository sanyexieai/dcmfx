#/bin/sh
#
# Runs all examples in this directory to check they work.

set -e

for dir in */; do
  echo ""
  echo "Testing $dir ..."

  cd "$dir"/gleam
  gleam format --check .
  gleam run --target erlang
  if [ "$dir" != "dicom_stream/" ]; then
    gleam run --target javascript
  fi

  cd ../rust
  cargo fmt --check
  cargo clippy -- --deny warnings
  cargo run

  cd ../..
done

echo ""
echo "Done"
