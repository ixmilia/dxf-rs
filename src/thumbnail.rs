use crate::code_pair_put_back::CodePairPutBack;

use crate::{CodePair, DxfError, DxfResult};

const FILE_HEADER_LENGTH: usize = 14;
const FILE_LENGTH_OFFSET: usize = 2;
const IMAGE_DATA_OFFSET_OFFSET: usize = 10;

const BITMAP_HEADER_PALETTE_COUNT_OFFSET: usize = 32;

pub(crate) fn read_thumbnail(iter: &mut CodePairPutBack) -> DxfResult<Option<image::DynamicImage>> {
    match read_thumbnail_bytes_from_code_pairs(iter)? {
        Some(mut data) => {
            if update_thumbnail_data_offset_in_situ(&mut data)? {
                read_thumbnail_from_bytes(&data)
            } else {
                Ok(None)
            }
        }
        None => Ok(None),
    }
}

fn read_thumbnail_bytes_from_code_pairs(iter: &mut CodePairPutBack) -> DxfResult<Option<Vec<u8>>> {
    // get the length; we don't really care about this since we'll just read whatever's there
    let _length = match iter.next() {
        Some(Ok(pair @ CodePair { code: 0, .. })) => {
            // likely 0/ENDSEC
            iter.put_back(Ok(pair));
            return Ok(None);
        }
        Some(Ok(pair @ CodePair { code: 90, .. })) => pair.assert_i32()? as usize,
        Some(Ok(pair)) => {
            return Err(DxfError::UnexpectedCode(pair.code, pair.offset));
        }
        Some(Err(e)) => return Err(e),
        None => return Ok(None),
    };

    // prepend the BMP header that always seems to be missing from DXF files
    let mut data: Vec<u8> = vec![
        b'B', b'M', // magic number
        0x00, 0x00, 0x00, 0x00, // file length (set below)
        0x00, 0x00, // reserved
        0x00, 0x00, // reserved
        0x00, 0x00, 0x00, 0x00, // image data offset (calculated elsewhere)
    ];

    // read the hex data
    loop {
        match iter.next() {
            Some(Ok(pair @ CodePair { code: 0, .. })) => {
                // likely 0/ENDSEC
                iter.put_back(Ok(pair));
                break;
            }
            Some(Ok(pair @ CodePair { code: 310, .. })) => {
                let line_data = pair.assert_binary()?;
                for b in line_data {
                    data.push(b);
                }
            }
            Some(Ok(pair)) => {
                return Err(DxfError::UnexpectedCode(pair.code, pair.offset));
            }
            Some(Err(e)) => return Err(e),
            None => break,
        }
    }

    let file_length = data.len();
    set_i32(&mut data, FILE_LENGTH_OFFSET, file_length as i32)?;
    Ok(Some(data))
}

fn update_thumbnail_data_offset_in_situ(data: &mut [u8]) -> DxfResult<bool> {
    // calculate the image data offset
    let dib_header_size = read_i32(data, FILE_HEADER_LENGTH)? as usize;

    // calculate the palette size
    let palette_size = if dib_header_size >= BITMAP_HEADER_PALETTE_COUNT_OFFSET + 4 {
        let palette_color_count = read_u32(
            data,
            FILE_HEADER_LENGTH + BITMAP_HEADER_PALETTE_COUNT_OFFSET,
        )? as usize;
        palette_color_count * 4 // always 4 bytes: BGRA
    } else {
        return Ok(false);
    };

    // set the image data offset
    let image_data_offset = FILE_HEADER_LENGTH + dib_header_size + palette_size;
    set_i32(data, IMAGE_DATA_OFFSET_OFFSET, image_data_offset as i32)?;

    Ok(true)
}

#[test]
fn set_thumbnail_offset_for_bitmapinfoheader_non_palette() {
    let mut data: Vec<u8> = vec![
        b'B', b'M', // magic number
        0x00, 0x00, 0x00, 0x00, // file length; not needed for this test
        0x00, 0x00, // reserved
        0x00, 0x00, // reserved
        0x00, 0x00, 0x00, 0x00, // the image data offset that will be filled in
        0x28, 0x00, 0x00, 0x00, // BITMAPINFOHEADER length
        0x00, 0x00, 0x00, 0x00, // width (not needed)
        0x00, 0x00, 0x00, 0x00, // height (not needed)
        0x00, 0x00, // color planes (not needed)
        0x00, 0x00, // bits per pixel (not needed)
        0x00, 0x00, 0x00, 0x00, // compression (not needed)
        0x00, 0x00, 0x00, 0x00, // image size (not needed)
        0x00, 0x00, 0x00, 0x00, // horizontal resolution (not needed)
        0x00, 0x00, 0x00, 0x00, // vertical resolution (not needed)
        0x00, 0x00, 0x00,
        0x00, // color palette count (0 means default of 2^n)
              // rest of struct not needed
    ];
    assert!(update_thumbnail_data_offset_in_situ(&mut data).unwrap());
    assert_eq!(0x36, read_i32(&data, IMAGE_DATA_OFFSET_OFFSET).unwrap());
}

#[test]
fn set_thumbnail_offset_for_bitmapinfoheader_palette_256() {
    let mut data: Vec<u8> = vec![
        b'B', b'M', // magic number
        0x00, 0x00, 0x00, 0x00, // file length; not needed for this test
        0x00, 0x00, // reserved
        0x00, 0x00, // reserved
        0x00, 0x00, 0x00, 0x00, // the image data offset that will be filled in
        0x28, 0x00, 0x00, 0x00, // BITMAPINFOHEADER length
        0x00, 0x00, 0x00, 0x00, // width (not needed)
        0x00, 0x00, 0x00, 0x00, // height (not needed)
        0x00, 0x00, // color planes (not needed)
        0x00, 0x00, // bits per pixel (not needed)
        0x00, 0x00, 0x00, 0x00, // compression (not needed)
        0x00, 0x00, 0x00, 0x00, // image size (not needed)
        0x00, 0x00, 0x00, 0x00, // horizontal resolution (not needed)
        0x00, 0x00, 0x00, 0x00, // vertical resolution (not needed)
        0x00, 0x01, 0x00,
        0x00, // color palette count (0 means default of 2^n)
              // rest of struct not needed
    ];
    assert!(update_thumbnail_data_offset_in_situ(&mut data).unwrap());
    assert_eq!(0x0436, read_i32(&data, IMAGE_DATA_OFFSET_OFFSET).unwrap());
}

#[test]
fn set_thumbnail_offset_for_bitmapv4header_non_palette() {
    let mut data: Vec<u8> = vec![
        b'B', b'M', // magic number
        0x00, 0x00, 0x00, 0x00, // file length; not needed for this test
        0x00, 0x00, // reserved
        0x00, 0x00, // reserved
        0x00, 0x00, 0x00, 0x00, // the image data offset that will be filled in
        0x6C, 0x00, 0x00, 0x00, // BITMAPV4HEADER length
        0x00, 0x00, 0x00, 0x00, // width (not needed)
        0x00, 0x00, 0x00, 0x00, // height (not needed)
        0x00, 0x00, // color planes (not needed)
        0x00, 0x00, // bits per pixel (not needed)
        0x00, 0x00, 0x00, 0x00, // compression (not needed)
        0x00, 0x00, 0x00, 0x00, // image size (not needed)
        0x00, 0x00, 0x00, 0x00, // horizontal resolution (not needed)
        0x00, 0x00, 0x00, 0x00, // vertical resolution (not needed)
        0x00, 0x00, 0x00,
        0x00, // color palette count (0 means default of 2^n)
              // rest of struct not needed
    ];
    assert!(update_thumbnail_data_offset_in_situ(&mut data).unwrap());
    assert_eq!(0x7A, read_i32(&data, IMAGE_DATA_OFFSET_OFFSET).unwrap());
}

#[test]
fn set_thumbnail_offset_for_bitmapv4header_palette_256() {
    let mut data: Vec<u8> = vec![
        b'B', b'M', // magic number
        0x00, 0x00, 0x00, 0x00, // file length; not needed for this test
        0x00, 0x00, // reserved
        0x00, 0x00, // reserved
        0x00, 0x00, 0x00, 0x00, // the image data offset that will be filled in
        0x6C, 0x00, 0x00, 0x00, // BITMAPV4HEADER length
        0x00, 0x00, 0x00, 0x00, // width (not needed)
        0x00, 0x00, 0x00, 0x00, // height (not needed)
        0x00, 0x00, // color planes (not needed)
        0x00, 0x00, // bits per pixel (not needed)
        0x00, 0x00, 0x00, 0x00, // compression (not needed)
        0x00, 0x00, 0x00, 0x00, // image size (not needed)
        0x00, 0x00, 0x00, 0x00, // horizontal resolution (not needed)
        0x00, 0x00, 0x00, 0x00, // vertical resolution (not needed)
        0x00, 0x01, 0x00,
        0x00, // color palette count (0 means default of 2^n)
              // rest of struct not needed
    ];
    assert!(update_thumbnail_data_offset_in_situ(&mut data).unwrap());
    assert_eq!(0x047A, read_i32(&data, IMAGE_DATA_OFFSET_OFFSET).unwrap());
}

fn read_thumbnail_from_bytes(data: &[u8]) -> DxfResult<Option<image::DynamicImage>> {
    let image = image::load_from_memory(data)?;
    Ok(Some(image))
}

fn read_i32(data: &[u8], offset: usize) -> DxfResult<i32> {
    let expected_length = offset + 4;
    if data.len() < expected_length {
        return Err(DxfError::UnexpectedEndOfInput);
    }

    let value = data[offset] as i32
        + ((data[offset + 1] as i32) << 8)
        + ((data[offset + 2] as i32) << 16)
        + ((data[offset + 3] as i32) << 24);
    Ok(value)
}

#[test]
fn test_read_i32() {
    let data: Vec<u8> = vec![0x00, 0x78, 0x56, 0x34, 0x12, 0x00];
    let value = read_i32(&data, 1).unwrap();
    assert_eq!(0x12345678, value);
}

fn read_u32(data: &[u8], offset: usize) -> DxfResult<u32> {
    let expected_length = offset + 4;
    if data.len() < expected_length {
        return Err(DxfError::UnexpectedEndOfInput);
    }

    let value = data[offset] as u32
        + ((data[offset + 1] as u32) << 8)
        + ((data[offset + 2] as u32) << 16)
        + ((data[offset + 3] as u32) << 24);
    Ok(value)
}

#[test]
fn test_get_u32() {
    let data: Vec<u8> = vec![0x00, 0x78, 0x56, 0x34, 0x12, 0x00];
    let value = read_u32(&data, 1).unwrap();
    assert_eq!(0x12345678, value);
}

fn set_i32(data: &mut [u8], offset: usize, value: i32) -> DxfResult<()> {
    let expected_length = offset + 4;
    if data.len() < expected_length {
        return Err(DxfError::UnexpectedEndOfInput);
    }

    data[offset] = value as u8;
    data[offset + 1] = (value >> 8) as u8;
    data[offset + 2] = (value >> 16) as u8;
    data[offset + 3] = (value >> 24) as u8;
    Ok(())
}

#[test]
fn test_set_i32() {
    let mut data = vec![0x00u8; 6];
    set_i32(&mut data, 1, 0x12345678).unwrap();
    assert_eq!(vec![0x00, 0x78, 0x56, 0x34, 0x12, 0x00], data);
}
