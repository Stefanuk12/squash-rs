pub type Result<T> = core::result::Result<T, Error>;
pub type CoreResult<T, E> = core::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Utf8(#[from] std::string::FromUtf8Error),
    #[error("expected a single character, found a string of length {0}")]
    CharTooLong(usize),
    #[error("expected a single character, found nothing")]
    CharMissing,
    #[error("value too large")]
    ValueTooLarge,
    #[error("invalid vlq: {0}")]
    InvalidVlq(u64),
    #[error("deserialize_any is not implemented")]
    DeserializeAnyNotImplemented,
    #[error("enum variant index {0} does not fit the single-byte tag")]
    VariantIndexTooLarge(u32),
    #[error("deserialization recursion depth limit exceeded")]
    RecursionDepthExceeded,
    #[error("failed to match variant")]
    DeserializeVariantNotMatched,
    #[error("{0}")]
    Custom(String),
}

#[cfg(feature = "serde")]
impl serde::ser::Error for Error {
    fn custom<T: core::fmt::Display>(msg: T) -> Self {
        Error::Custom(msg.to_string())
    }
}
#[cfg(feature = "serde")]
impl serde::de::Error for Error {
    fn custom<T: core::fmt::Display>(msg: T) -> Self {
        Error::Custom(msg.to_string())
    }
}
