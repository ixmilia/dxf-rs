use std::io::Write;

use byteorder::{LittleEndian, WriteBytesExt};

use crate::code_pair_value::{escape_control_characters, escape_unicode_to_ascii};
use crate::enums::AcadVersion;
use crate::{CodePair, CodePairValue, DxfResult};

pub(crate) struct CodePairWriter<'a, T>
where
    T: Write + ?Sized + 'a,
{
    writer: &'a mut T,
    as_text: bool,
    text_as_ascii: bool,
    version: AcadVersion,
}

impl<'a, T: Write + ?Sized> CodePairWriter<'a, T> {
    pub fn new(
        writer: &'a mut T,
        as_text: bool,
        text_as_ascii: bool,
        version: AcadVersion,
    ) -> Self {
        CodePairWriter {
            writer,
            as_text,
            text_as_ascii,
            version,
        }
    }
    pub fn write_prelude(&mut self) -> DxfResult<()> {
        if !self.as_text {
            self.writer
                .write_fmt(format_args!("AutoCAD Binary DXF\r\n"))?;
            self.writer.write_u8(0x1A)?;
            self.writer.write_u8(0x00)?;
        }

        Ok(())
    }
    pub fn write_code_pair(&mut self, pair: &CodePair) -> DxfResult<()> {
        if self.as_text {
            self.write_ascii_code_pair(pair)
        } else {
            self.write_binary_code_pair(pair)
        }
    }
    fn write_ascii_code_pair(&mut self, pair: &CodePair) -> DxfResult<()> {
        self.writer
            .write_fmt(format_args!("{: >3}\r\n", pair.code))?;
        match pair.value {
            CodePairValue::Str(ref s) => {
                let s = escape_control_characters(s);
                let s = if self.text_as_ascii {
                    escape_unicode_to_ascii(&s)
                } else {
                    s
                };
                self.writer.write_fmt(format_args!("{}\r\n", s))?;
            }
            _ => self.writer.write_fmt(format_args!("{}\r\n", &pair.value))?,
        };
        Ok(())
    }
    fn write_binary_code_pair(&mut self, pair: &CodePair) -> DxfResult<()> {
        // write code
        if self.version >= AcadVersion::R13 {
            self.writer.write_i16::<LittleEndian>(pair.code as i16)?;
        } else if pair.code >= 255 {
            self.writer.write_u8(255)?;
            self.writer.write_i16::<LittleEndian>(pair.code as i16)?;
        } else {
            self.writer.write_u8(pair.code as u8)?;
        }

        // write value
        match pair.value {
            CodePairValue::Boolean(s) => {
                if self.version >= AcadVersion::R13 {
                    self.writer.write_u8(s as u8)?
                } else {
                    self.writer.write_i16::<LittleEndian>(s)?
                }
            }
            CodePairValue::Integer(i) => self.writer.write_i32::<LittleEndian>(i)?,
            CodePairValue::Long(l) => self.writer.write_i64::<LittleEndian>(l)?,
            CodePairValue::Short(s) => self.writer.write_i16::<LittleEndian>(s)?,
            CodePairValue::Double(d) => self.writer.write_f64::<LittleEndian>(d)?,
            CodePairValue::Str(ref s) => {
                for &b in escape_control_characters(s).as_bytes() {
                    self.writer.write_u8(b)?;
                }

                self.writer.write_u8(0)?;
            }
            CodePairValue::Binary(ref buf) => {
                self.writer.write_u8(buf.len() as u8)?;
                for b in buf {
                    self.writer.write_u8(*b)?;
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::code_pair_writer::CodePairWriter;
    use crate::enums::AcadVersion;
    use crate::CodePair;
    use std::io::{BufRead, BufReader, Cursor, Seek, SeekFrom};

    fn write_in_binary(pair: &CodePair) -> Vec<u8> {
        let mut buf = Cursor::new(vec![]);
        let mut writer = CodePairWriter {
            writer: &mut buf,
            as_text: false,
            text_as_ascii: true,
            version: AcadVersion::R2004,
        };
        writer
            .write_binary_code_pair(pair)
            .expect("expected write to succeed");
        buf.seek(SeekFrom::Start(0))
            .expect("expected seek to succeed");
        buf.into_inner()
    }

    fn write_in_ascii(pair: &CodePair) -> String {
        let mut buf = Cursor::new(vec![]);
        let mut writer = CodePairWriter {
            writer: &mut buf,
            as_text: true,
            text_as_ascii: true,
            version: AcadVersion::R2004,
        };
        writer
            .write_ascii_code_pair(pair)
            .expect("expected write to succeed");
        buf.seek(SeekFrom::Start(0))
            .expect("expected seek to succeed");
        let reader = BufReader::new(&mut buf);
        reader
            .lines()
            .map(|l| l.unwrap())
            .fold(String::new(), |a, l| a + l.as_str() + "\r\n")
    }

    #[test]
    fn write_string_in_binary() {
        let pair = CodePair::new_str(1, "A");
        let actual = write_in_binary(&pair);

        // code 0x0001, value 0x41 = "A", NUL
        let expected: Vec<u8> = vec![0x01, 0x00, 0x41, 0x00];
        assert_eq!(expected, actual);
    }

    #[test]
    fn write_binary_chunk_in_binary() {
        let pair = CodePair::new_binary(310, vec![0x01, 0x02]);
        let actual = write_in_binary(&pair);

        // code 0x136, length 2, data [0x01, 0x02]
        let expected: Vec<u8> = vec![0x36, 0x01, 0x02, 0x01, 0x02];
        assert_eq!(expected, actual);
    }

    #[test]
    fn write_binary_chunk_in_ascii() {
        let pair = CodePair::new_binary(310, vec![0x01, 0x02]);
        let actual = write_in_ascii(&pair);
        let expected = "310\r\n0102\r\n";
        assert_eq!(expected, actual);
    }

    #[test]
    fn write_code_450_in_binary() {
        let pair = CodePair::new_i32(450, 37);
        let actual = write_in_binary(&pair);

        // code 450 = 0x1C2, value = 37 (0x25)
        assert_eq!(vec![0xC2, 0x01, 0x25, 0x00, 0x00, 0x00], actual);
    }

    #[test]
    fn write_unicode_in_ascii() {
        let pair = CodePair::new_str(1, "АаЯя");
        let actual = write_in_ascii(&pair);
        let expected = "  1\r\n\\U+0410\\U+0430\\U+042F\\U+044F\r\n";
        assert_eq!(expected, actual);
    }

    #[test]
    fn write_aligned_code_pair_in_ascii_1() {
        let pair = CodePair::new_str(1, "A");
        let actual = write_in_ascii(&pair);
        let expected = "  1\r\nA\r\n";
        assert_eq!(expected, actual);
    }

    #[test]
    fn write_aligned_code_pair_in_ascii_2() {
        let pair = CodePair::new_str(100, "A");
        let actual = write_in_ascii(&pair);
        let expected = "100\r\nA\r\n";
        assert_eq!(expected, actual);
    }
}
