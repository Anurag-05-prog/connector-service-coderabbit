//! Errors and error specific types for universal use

use crate::types::MinorUnit;

/// Custom Result
/// A custom datatype that wraps the error variant <E> into a report, allowing
/// error_stack::Report<E> specific extendability
///
/// Effectively, equivalent to `Result<T, error_stack::Report<E>>`
pub type CustomResult<T, E> = error_stack::Result<T, E>;

/// Parsing Errors
#[allow(missing_docs)] // Only to prevent warnings about struct fields not being documented
#[derive(Debug, thiserror::Error)]
pub enum ParsingError {
    ///Failed to parse enum
    #[error("Failed to parse enum: {0}")]
    EnumParseFailure(&'static str),
    ///Failed to parse struct
    #[error("Failed to parse struct: {0}")]
    StructParseFailure(&'static str),
    /// Failed to encode data to given format
    #[error("Failed to serialize to {0} format")]
    EncodeError(&'static str),
    /// Failed to parse data
    #[error("Unknown error while parsing")]
    UnknownError,
    /// Failed to parse datetime
    #[error("Failed to parse datetime")]
    DateTimeParsingError,
    /// Failed to parse email
    #[error("Failed to parse email")]
    EmailParsingError,
    /// Failed to parse phone number
    #[error("Failed to parse phone number")]
    PhoneNumberParsingError,
    /// Failed to parse Float value for converting to decimal points
    #[error("Failed to parse Float value for converting to decimal points")]
    FloatToDecimalConversionFailure,
    /// Failed to parse Decimal value for i64 value conversion
    #[error("Failed to parse Decimal value for i64 value conversion")]
    DecimalToI64ConversionFailure,
    /// Failed to parse string value for f64 value conversion
    #[error("Failed to parse string value for f64 value conversion")]
    StringToFloatConversionFailure,
    /// Failed to parse i64 value for f64 value conversion
    #[error("Failed to parse i64 value for f64 value conversion")]
    I64ToDecimalConversionFailure,
    /// Failed to parse String value to Decimal value conversion because `error`
    #[error("Failed to parse String value to Decimal value conversion because {error}")]
    StringToDecimalConversionFailure { error: String },
    /// Failed to convert the given integer because of integer overflow error
    #[error("Integer Overflow error")]
    IntegerOverflow,
}

/// Validation errors.
#[allow(missing_docs)] // Only to prevent warnings about struct fields not being documented
#[derive(Debug, thiserror::Error, Clone, PartialEq)]
pub enum ValidationError {
    /// The provided input is missing a required field.
    #[error("Missing required field: {field_name}")]
    MissingRequiredField { field_name: String },

    /// An incorrect value was provided for the field specified by `field_name`.
    #[error("Incorrect value provided for field: {field_name}")]
    IncorrectValueProvided { field_name: &'static str },

    /// An invalid input was provided.
    #[error("{message}")]
    InvalidValue { message: String },
}

/// Api Models construction error
#[derive(Debug, Clone, thiserror::Error, PartialEq)]
pub enum PercentageError {
    /// Percentage Value provided was invalid
    #[error("Invalid Percentage value")]
    InvalidPercentageValue,

    /// Error occurred while calculating percentage
    #[error("Failed apply percentage of {percentage} on {amount}")]
    UnableToApplyPercentage {
        /// percentage value
        percentage: f32,
        /// amount value
        amount: MinorUnit,
    },
}

/// Allow [error_stack::Report] to convert between error types
pub trait ErrorSwitch<T> {
    /// Get the next error type that the source error can be escalated into
    /// This does not consume the source error since we need to keep it in context
    fn switch(&self) -> T;
}

/// Allow [error_stack::Report] to convert between error types
/// This serves as an alternative to [ErrorSwitch]
pub trait ErrorSwitchFrom<T> {
    /// Convert to an error type that the source can be escalated into
    /// This does not consume the source error since we need to keep it in context
    fn switch_from(error: &T) -> Self;
}

impl<T, S> ErrorSwitch<T> for S
where
    T: ErrorSwitchFrom<Self>,
{
    fn switch(&self) -> T {
        T::switch_from(self)
    }
}

/// Allows [error_stack::Report] to change between error contexts
/// using the dependent [ErrorSwitch] trait to define relations & mappings between traits
pub trait ReportSwitchExt<T, U> {
    /// Switch to the intended report by calling switch
    /// requires error switch to be already implemented on the error type
    fn switch(self) -> Result<T, error_stack::Report<U>>;
}

impl<T, U, V> ReportSwitchExt<T, U> for Result<T, error_stack::Report<V>>
where
    V: ErrorSwitch<U> + error_stack::Context,
    U: error_stack::Context,
{
    #[track_caller]
    fn switch(self) -> Result<T, error_stack::Report<U>> {
        match self {
            Ok(i) => Ok(i),
            Err(er) => {
                let new_c = er.current_context().switch();
                Err(er.change_context(new_c))
            }
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CryptoError {
    /// The cryptographic algorithm was unable to encode the message
    #[error("Failed to encode given message")]
    EncodingFailed,
    /// The cryptographic algorithm was unable to decode the message
    #[error("Failed to decode given message")]
    DecodingFailed,
    /// The cryptographic algorithm was unable to sign the message
    #[error("Failed to sign message")]
    MessageSigningFailed,
    /// The cryptographic algorithm was unable to verify the given signature
    #[error("Failed to verify signature")]
    SignatureVerificationFailed,
    /// The provided key length is invalid for the cryptographic algorithm
    #[error("Invalid key length")]
    InvalidKeyLength,
    /// The provided IV length is invalid for the cryptographic algorithm
    #[error("Invalid IV length")]
    InvalidIvLength,
}

impl ErrorSwitchFrom<common_enums::CurrencyError> for ParsingError {
    fn switch_from(error: &common_enums::CurrencyError) -> Self {
        match error {
            common_enums::CurrencyError::UnsupportedCurrency { .. } => {
                ParsingError::StructParseFailure("currency decimal configuration")
            }
        }
    }
}

/// Integrity check errors.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct IntegrityCheckError {
    /// Field names for which integrity check failed!
    pub field_names: String,
    /// Connector transaction reference id
    pub connector_transaction_id: Option<String>,
}
