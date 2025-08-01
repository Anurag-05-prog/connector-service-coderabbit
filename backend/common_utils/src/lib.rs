//! Common utilities for connector service

pub mod crypto;
pub mod custom_serde;
pub mod errors;
pub mod ext_traits;
pub mod fp_utils;
pub mod id_type;
pub mod macros;
pub mod new_types;
pub mod pii;
pub mod request;
pub mod types;
// Re-export commonly used items
pub use errors::{CustomResult, ParsingError, ValidationError};
pub use id_type::{CustomerId, MerchantId};
pub use pii::{Email, SecretSerdeValue};
pub use request::{Method, Request, RequestContent};
pub use types::{
    AmountConvertor, FloatMajorUnit, FloatMajorUnitForConnector, MinorUnit, MinorUnitForConnector,
    StringMajorUnit, StringMajorUnitForConnector, StringMinorUnit,
};
pub mod events;
pub mod global_id;

pub mod consts;

fn generate_ref_id_with_default_length<const MAX_LENGTH: u8, const MIN_LENGTH: u8>(
    prefix: &str,
) -> id_type::LengthId<MAX_LENGTH, MIN_LENGTH> {
    id_type::LengthId::<MAX_LENGTH, MIN_LENGTH>::new(prefix)
}

/// Generate a time-ordered (time-sortable) unique identifier using the current time
#[inline]
pub fn generate_time_ordered_id(prefix: &str) -> String {
    format!("{prefix}_{}", uuid::Uuid::now_v7().as_simple())
}

pub mod date_time {
    #[cfg(feature = "async_ext")]
    use std::time::Instant;
    use std::{marker::PhantomData, num::NonZeroU8};

    use hyperswitch_masking::{Deserialize, Serialize};
    use time::{
        format_description::{
            well_known::iso8601::{Config, EncodedConfig, Iso8601, TimePrecision},
            BorrowedFormatItem,
        },
        OffsetDateTime, PrimitiveDateTime,
    };

    /// Enum to represent date formats
    #[derive(Debug)]
    pub enum DateFormat {
        /// Format the date in 20191105081132 format
        YYYYMMDDHHmmss,
        /// Format the date in 20191105 format
        YYYYMMDD,
        /// Format the date in 201911050811 format
        YYYYMMDDHHmm,
        /// Format the date in 05112019081132 format
        DDMMYYYYHHmmss,
    }

    /// Create a new [`PrimitiveDateTime`] with the current date and time in UTC.
    pub fn now() -> PrimitiveDateTime {
        let utc_date_time = OffsetDateTime::now_utc();
        PrimitiveDateTime::new(utc_date_time.date(), utc_date_time.time())
    }

    /// Convert from OffsetDateTime to PrimitiveDateTime
    pub fn convert_to_pdt(offset_time: OffsetDateTime) -> PrimitiveDateTime {
        PrimitiveDateTime::new(offset_time.date(), offset_time.time())
    }

    /// Return the UNIX timestamp of the current date and time in UTC
    pub fn now_unix_timestamp() -> i64 {
        OffsetDateTime::now_utc().unix_timestamp()
    }

    /// Calculate execution time for a async block in milliseconds
    #[cfg(feature = "async_ext")]
    pub async fn time_it<T, Fut: futures::Future<Output = T>, F: FnOnce() -> Fut>(
        block: F,
    ) -> (T, f64) {
        let start = Instant::now();
        let result = block().await;
        (result, start.elapsed().as_secs_f64() * 1000f64)
    }

    /// Return the given date and time in UTC with the given format Eg: format: YYYYMMDDHHmmss Eg: 20191105081132
    pub fn format_date(
        date: PrimitiveDateTime,
        format: DateFormat,
    ) -> Result<String, time::error::Format> {
        let format = <&[BorrowedFormatItem<'_>]>::from(format);
        date.format(&format)
    }

    /// Return the current date and time in UTC with the format [year]-[month]-[day]T[hour]:[minute]:[second].mmmZ Eg: 2023-02-15T13:33:18.898Z
    pub fn date_as_yyyymmddthhmmssmmmz() -> Result<String, time::error::Format> {
        const ISO_CONFIG: EncodedConfig = Config::DEFAULT
            .set_time_precision(TimePrecision::Second {
                decimal_digits: NonZeroU8::new(3),
            })
            .encode();
        now().assume_utc().format(&Iso8601::<ISO_CONFIG>)
    }

    impl From<DateFormat> for &[BorrowedFormatItem<'_>] {
        fn from(format: DateFormat) -> Self {
            match format {
                DateFormat::YYYYMMDDHHmmss => time::macros::format_description!("[year repr:full][month padding:zero repr:numerical][day padding:zero][hour padding:zero repr:24][minute padding:zero][second padding:zero]"),
                DateFormat::YYYYMMDD => time::macros::format_description!("[year repr:full][month padding:zero repr:numerical][day padding:zero]"),
                DateFormat::YYYYMMDDHHmm => time::macros::format_description!("[year repr:full][month padding:zero repr:numerical][day padding:zero][hour padding:zero repr:24][minute padding:zero]"),
                DateFormat::DDMMYYYYHHmmss => time::macros::format_description!("[day padding:zero][month padding:zero repr:numerical][year repr:full][hour padding:zero repr:24][minute padding:zero][second padding:zero]"),
            }
        }
    }

    /// Format the date in 05112019 format
    #[derive(Debug, Clone)]
    pub struct DDMMYYYY;
    /// Format the date in 20191105 format
    #[derive(Debug, Clone)]
    pub struct YYYYMMDD;
    /// Format the date in 20191105081132 format
    #[derive(Debug, Clone)]
    pub struct YYYYMMDDHHmmss;

    /// To serialize the date in Dateformats like YYYYMMDDHHmmss, YYYYMMDD, DDMMYYYY
    #[derive(Debug, Deserialize, Clone)]
    pub struct DateTime<T: TimeStrategy> {
        inner: PhantomData<T>,
        value: PrimitiveDateTime,
    }

    impl<T: TimeStrategy> From<PrimitiveDateTime> for DateTime<T> {
        fn from(value: PrimitiveDateTime) -> Self {
            Self {
                inner: PhantomData,
                value,
            }
        }
    }

    /// Time strategy for the Date, Eg: YYYYMMDDHHmmss, YYYYMMDD, DDMMYYYY
    pub trait TimeStrategy {
        /// Stringify the date as per the Time strategy
        fn fmt(input: &PrimitiveDateTime, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result;
    }

    impl<T: TimeStrategy> Serialize for DateTime<T> {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            serializer.collect_str(self)
        }
    }

    impl<T: TimeStrategy> std::fmt::Display for DateTime<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            T::fmt(&self.value, f)
        }
    }

    impl TimeStrategy for DDMMYYYY {
        fn fmt(input: &PrimitiveDateTime, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let year = input.year();
            #[allow(clippy::as_conversions)]
            let month = input.month() as u8;
            let day = input.day();
            let output = format!("{day:02}{month:02}{year}");
            f.write_str(&output)
        }
    }

    impl TimeStrategy for YYYYMMDD {
        fn fmt(input: &PrimitiveDateTime, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let year = input.year();
            #[allow(clippy::as_conversions)]
            let month: u8 = input.month() as u8;
            let day = input.day();
            let output = format!("{year}{month:02}{day:02}");
            f.write_str(&output)
        }
    }

    impl TimeStrategy for YYYYMMDDHHmmss {
        fn fmt(input: &PrimitiveDateTime, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let year = input.year();
            #[allow(clippy::as_conversions)]
            let month = input.month() as u8;
            let day = input.day();
            let hour = input.hour();
            let minute = input.minute();
            let second = input.second();
            let output = format!("{year}{month:02}{day:02}{hour:02}{minute:02}{second:02}");
            f.write_str(&output)
        }
    }
}
