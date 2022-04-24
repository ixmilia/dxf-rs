// Due to https://github.com/rust-analyzer/rust-analyzer/issues/4075, rust-analyzer can't navigate to relative `mod`
// statements.  To make working with rust-analyzer easier in this repo, we'll manually specify the modules.

//include!(concat!(env!("OUT_DIR"), "/generated/mod.rs"));

#[allow(clippy::all)]
pub mod entities {
    include!(concat!(env!("OUT_DIR"), "/generated/entities.rs"));
}

#[allow(clippy::all)]
pub mod header {
    include!(concat!(env!("OUT_DIR"), "/generated/header.rs"));
}

#[allow(clippy::all)]
pub mod objects {
    include!(concat!(env!("OUT_DIR"), "/generated/objects.rs"));
}

#[allow(clippy::all)]
pub mod tables {
    include!(concat!(env!("OUT_DIR"), "/generated/tables.rs"));
}
