use snafu::Snafu;

use super::Literal;

#[derive(Snafu, Debug)]
pub enum ExpressionError {
    #[snafu(display("invalid resource: {:?}", resource))]
    InvalidResource { resource: Literal },
    #[snafu(display("resource: {:?} not found", resource))]
    ResourceNotFound { resource: String },
}
