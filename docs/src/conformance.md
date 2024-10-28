# DICOM Conformance

DCMfx reads all valid DICOM Part 10 (P10) data, as well as many DICOM files that
that don't strictly conform to the DICOM P10 standard.

When writing DICOM P10 data, strict conformance of the data being written is
_not_ enforced, because any DICOM P10 data that was able to be _read_ should
also be able to be _written_, even if parts of it were non-conformant in
some way. Still, many variants of invalid DICOM P10 data or files will be
corrected by going through a read/write cycle in DCMfx.

## UTF-8 Conversion

DCMfx converts all strings to UTF-8 when reading DICOM P10 data. This is because
native DICOM string data is complex to work with, and UTF-8 is the preferred
string encoding of modern systems.

All Specific Character Sets defined by the DICOM standard are supported,
including the use of Code Extensions via ISO 2022 escape sequences. If an
invalid byte is encountered during UTF-8 conversion then it is converted to the
� (U+FFFD) character.

Due to the UTF-8 conversion, DICOM P10 data written by this library always uses
the `ISO_IR 192` (UTF-8) Specific Character Set.

## Sequences and Items of Undefined Length

DCMfx converts sequences and items that have defined lengths to use undefined
lengths with explicit delimiters. This consumes slightly more space, which may
be noticeable for data sets with a very large number of sequences or items, but
is necessary in order to be able to stream DICOM P10 data in a memory-efficient
way.

## DICOM JSON

DCMfx supports conversion to and from the DICOM JSON Model. The following
details are relevant to users of this feature:

1. The DICOM JSON specification is optionally extended to allow encapsulated
   pixel data to be stored. This is done by encoding the binary data present in
   the '(7FE0,0010) PixelData' data element in Base64. This matches the behavior
   of other libraries such as [`pydicom`](https://pydicom.github.io/).

2. `BulkDataURI` specifiers for storing and retrieving data from external
   sources are not supported. Binary data must be encoded inline using Base64.

3. Because floating point `Infinity`, `-Infinity`, and `NaN` are not supported
   by JSON's `number` type, DCMfx stores them as quoted strings: `"Infinity"`,
   `"-Infinity"`, and `"NaN"`.

4. 64-bit integer values outside the range representable accurately by
   JavaScript's `number` type are stored as quoted strings to avoid any loss of
   precision.
