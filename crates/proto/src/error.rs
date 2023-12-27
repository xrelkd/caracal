use std::borrow::Cow;

use snafu::Snafu;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum UnexpectedDataFormatError {
    #[snafu(display("Unexpected data format: unknown value `{value}`",))]
    UnknownValue { value: Cow<'static, str> },

    #[snafu(display("Unexpected data format: missing field `{field}`",))]
    MissingField { field: Cow<'static, str> },

    #[snafu(display("Unexpected data format, error: {}", source))]
    UnexpectedFormat { source: Box<dyn std::error::Error + Send + Sync> },
}
