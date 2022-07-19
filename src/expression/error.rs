use snafu::Snafu;

use super::Literal;

#[derive(Snafu, Debug)]
pub enum ExprError {
    #[snafu(display("invalid resource: {:?}", resource))]
    InvalidResource { resource: Literal },
    #[snafu(display("resource: {:?} not found", resource))]
    ResourceNotFound { resource: String },
}
