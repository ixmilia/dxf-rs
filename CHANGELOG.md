Changelog
=========

## 0.6.0

- Improvements writing files for `R13`+.
- Handle alternate header values for `$ACADMAINTVER`.
- Improved handling of unexpected code pairs.
- Update various dependencies.

## 0.5.0

- Update to Rust 2018 edition.
- Improve entity handle consistency.
- Migrate to MIT license.
- Improve compatibility with non-standard code pair values.
- Improve `THUMBNAILIMAGE` compatibility.

## 0.4.0

- Add support for R2018 drawings.
- Improve compatability when writing files.
- Improve non-ASCII text encoding handling.
- Support post R13 binary files.

## 0.3.0

- Fall back to default values when enum parsing fails.
- Track file offsets when reading yielding more actionable errors.
- Allow parsing of both Windows-style and standard UUIDs.
- Write trailing attributes on `INSERT` entities.
- Read/write `ATTRIB`/`ATTDEF` with attached `MTEXT` entities.
- Add optional [serde](https://github.com/serde-rs/serde) support via the `serialize` feature flag.
- Remove dependency on deprecated `time` crate.

## 0.2.1

- Use `image` crate for thumbnail images.
- Fix incorrect min/max versions for objects.
- Enable following pointers and object handles, including setting handles on file write.
- Remove `Sized` constraint when reading and writing.

## 0.2.0

- Add support for reading/writing binary files.
- Add support for reading/writing DXB binary files.
- Enable blocks and classes.
- Add support for `OBJECTS` section.
- Add support for `DIMENSION` entities.
- Support all forms of extended data.
- Add `.clear()` and `.normalize()` methods to `Drawing` struct.

## 0.1.1

- Properly expose fields on `Point` and `Vector` structs.

## 0.1.0

- Support reading/writing ASCII files.
- Simple entity support.
- Tables support (layers, etc.)
