// Copyright (c) IxMilia.  All Rights Reserved.  Licensed under the Apache License, Version 2.0.  See License.txt in the project root for license information.

// other implementation is in `generated/header.rs`

use std::io::Write;
use itertools::PutBack;

use ::{
    CodePair,
    DxfError,
    DxfResult,
};
use code_pair_writer::CodePairWriter;

pub use generated::header::*;

impl Header {
    #[doc(hidden)]
    pub fn read<I>(iter: &mut PutBack<I>) -> DxfResult<Header>
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
                            let last_header_variable = try!(pair.value.assert_string());
                            loop {
                                match iter.next() {
                                    Some(Ok(pair)) => {
                                        if pair.code == 0 || pair.code == 9 {
                                            // ENDSEC or a new header variable
                                            iter.put_back(Ok(pair));
                                            break;
                                        }
                                        else {
                                            try!(header.set_header_value(&last_header_variable, &pair));
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
    #[doc(hidden)]
    pub fn write<T>(&self, writer: &mut CodePairWriter<T>) -> DxfResult<()>
        where T: Write {

        try!(writer.write_code_pair(&CodePair::new_str(0, "SECTION")));
        try!(writer.write_code_pair(&CodePair::new_str(2, "HEADER")));
        try!(self.write_code_pairs(writer));
        try!(writer.write_code_pair(&CodePair::new_str(0, "ENDSEC")));
        Ok(())
    }
}
