/// Represents the expected data type of a `CodePair`.
#[derive(PartialEq)]
pub enum ExpectedType {
    Boolean,
    Integer,
    Long,
    Short,
    Double,
    Str,
    Binary,
}

impl ExpectedType {
    /// Returns an option of the `ExpectedType` for the given code, or `None`.
    pub fn expected_type(code: i32) -> ::std::option::Option<Self> {
        match code {
            0..=9 => Some(ExpectedType::Str),
            10..=39 => Some(ExpectedType::Double),
            40..=59 => Some(ExpectedType::Double),
            60..=79 => Some(ExpectedType::Short),
            90..=99 => Some(ExpectedType::Integer),
            100..=102 => Some(ExpectedType::Str),
            105 => Some(ExpectedType::Str),
            110..=119 => Some(ExpectedType::Double),
            120..=129 => Some(ExpectedType::Double),
            130..=139 => Some(ExpectedType::Double),
            140..=149 => Some(ExpectedType::Double),
            160..=169 => Some(ExpectedType::Long),
            170..=179 => Some(ExpectedType::Short),
            210..=239 => Some(ExpectedType::Double),
            270..=279 => Some(ExpectedType::Short),
            280..=289 => Some(ExpectedType::Short),
            290..=299 => Some(ExpectedType::Boolean),
            300..=309 => Some(ExpectedType::Str),
            310..=319 => Some(ExpectedType::Binary),
            320..=329 => Some(ExpectedType::Str),
            330..=369 => Some(ExpectedType::Str),
            370..=379 => Some(ExpectedType::Short),
            380..=389 => Some(ExpectedType::Short),
            390..=399 => Some(ExpectedType::Str),
            400..=409 => Some(ExpectedType::Short),
            410..=419 => Some(ExpectedType::Str),
            420..=429 => Some(ExpectedType::Integer),
            430..=439 => Some(ExpectedType::Str),
            440..=449 => Some(ExpectedType::Integer),
            450..=459 => Some(ExpectedType::Integer),
            460..=469 => Some(ExpectedType::Double),
            470..=479 => Some(ExpectedType::Str),
            480..=481 => Some(ExpectedType::Str),
            999 => Some(ExpectedType::Str),
            1000..=1009 => Some(ExpectedType::Str),
            1010..=1059 => Some(ExpectedType::Double),
            1060..=1070 => Some(ExpectedType::Short),
            1071 => Some(ExpectedType::Integer),
            _ => None,
        }
    }
}
