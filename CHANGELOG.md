Changelog
=========

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
