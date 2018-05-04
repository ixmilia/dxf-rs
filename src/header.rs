// Copyright (c) IxMilia.  All Rights Reserved.  Licensed under the Apache License, Version 2.0.  See License.txt in the project root for license information.

// other implementation is in `generated/header.rs`

use std::io::Write;
use itertools::PutBack;

use ::{
    CodePair,
    DxfError,
    DxfResult,
};
use ::helper_functions::*;
use code_pair_writer::CodePairWriter;

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
    pub(crate) fn read<I>(iter: &mut PutBack<I>) -> DxfResult<Header>
        where I: Iterator<Item = DxfResult<CodePair>> {

        let mut header = Header::default();
        loop {
            match iter.next() {
                Some(Ok(pair)) => {
                    match pair.code {
                        0 => {
                            iter.put_back(Ok(pair));
                            break;
                        },
                        9 => {
                            let last_header_variable = pair.assert_string()?;
                            loop {
                                match iter.next() {
                                    Some(Ok(pair)) => {
                                        if pair.code == 0 || pair.code == 9 {
                                            // ENDSEC or a new header variable
                                            iter.put_back(Ok(pair));
                                            break;
                                        }
                                        else {
                                            header.set_header_value(&last_header_variable, &pair)?;
                                        }
                                    },
                                    Some(Err(e)) => return Err(e),
                                    None => break,
                                }
                            }
                        },
                        _ => return Err(DxfError::UnexpectedCodePair(pair, String::from(""))),
                    }
                },
                Some(Err(e)) => return Err(e),
                None => break,
            }
        }

        Ok(header)
    }
    pub(crate) fn write<T>(&self, writer: &mut CodePairWriter<T>, next_available_handle: u32) -> DxfResult<()>
        where T: Write + ?Sized {

        writer.write_code_pair(&CodePair::new_str(0, "SECTION"))?;
        writer.write_code_pair(&CodePair::new_str(2, "HEADER"))?;
        self.write_code_pairs(writer, next_available_handle)?;
        writer.write_code_pair(&CodePair::new_str(0, "ENDSEC"))?;
        Ok(())
    }
}
