## These are big-item issues that don't make sense putting in the actual code

# io::Result
io::Result is commonly used for error handling during parsing.  The rust-ideomatic way of
handling errors is to use *your own* error enum that *wraps* existing errors if they crop up.

This way, you'll have `io::Error` *only* for actual read/write errors, and a more detailed error
enum for your own things.

```rust
pub type DfxResult<T> = Result<T, DfxError>;
pub enum DfxError {
    IoError(io::Error),
    DataParseError(...),
    InvalidCodePair(...),
    OperandWithoutFoo(...),
    BarIsntPresent(...),
}

impl From<io::Error> for DfxError {
    fn from(ioe: io::Error) -> DfxError {
        DfxError::IoError(ioe)
    }
}

impl std::error::Error for DfxError {
    // lots of work goes here (sadly)
}
```