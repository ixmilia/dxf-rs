// Copyright (c) IxMilia.  All Rights Reserved.  Licensed under the Apache License, Version 2.0.  See License.txt in the project root for license information.

// other implementation is in `generated/header.rs`

use std::io::{Read, Write};

use code_pair_put_back::CodePairPutBack;
use code_pair_writer::CodePairWriter;
use enums::*;
use helper_functions::*;
use {CodePair, DxfError, DxfResult};

pub use generated::header::*;

impl Header {
    /// Ensure all values are valid.
    pub fn normalize(&mut self) {
        ensure_positive_or_default(&mut self.default_text_height, 0.2);
        ensure_positive_or_default(&mut self.trace_width, 0.05);
        default_if_empty(&mut self.text_style, "STANDARD");
        default_if_empty(&mut self.current_layer, "0");
        default_if_empty(&mut self.current_entity_line_type, "BYLAYER");
        default_if_empty(&mut self.dimension_style_name, "STANDARD");
        default_if_empty(&mut self.file_name, ".");
    }
    pub(crate) fn read<T>(iter: &mut CodePairPutBack<T>) -> DxfResult<Header>
    where
        T: Read,
    {
        let mut header = Header::default();
        loop {
            match iter.next() {
                Some(Ok(pair)) => {
                    match pair.code {
                        0 => {
                            iter.put_back(Ok(pair));
                            break;
                        }
                        9 => {
                            let last_header_variable = pair.assert_string()?;
                            loop {
                                match iter.next() {
                                    Some(Ok(pair)) => {
                                        if pair.code == 0 || pair.code == 9 {
                                            // ENDSEC or a new header variable
                                            iter.put_back(Ok(pair));
                                            break;
                                        } else {
                                            header
                                                .set_header_value(&last_header_variable, &pair)?;
                                            if last_header_variable == "$ACADVER"
                                                && header.version >= AcadVersion::R2007
                                            {
                                                iter.read_as_utf8();
                                            }
                                        }
                                    }
                                    Some(Err(e)) => return Err(e),
                                    None => break,
                                }
                            }
                        }
                        _ => return Err(DxfError::UnexpectedCodePair(pair, String::from(""))),
                    }
                }
                Some(Err(e)) => return Err(e),
                None => break,
            }
        }

        Ok(header)
    }
    pub(crate) fn write<T>(
        &self,
        writer: &mut CodePairWriter<T>,
        next_available_handle: u32,
    ) -> DxfResult<()>
    where
        T: Write + ?Sized,
    {
        writer.write_code_pair(&CodePair::new_str(0, "SECTION"))?;
        writer.write_code_pair(&CodePair::new_str(2, "HEADER"))?;
        self.write_code_pairs(writer, next_available_handle)?;
        writer.write_code_pair(&CodePair::new_str(0, "ENDSEC"))?;
        Ok(())
    }
}
