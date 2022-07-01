use crate::context::Context;
use crate::source::Table;

use super::error::ExpressionError;
use super::Expression;

impl Expression for Table {
    #[inline]
    fn evaluate(&self, _context: &mut Context, _args: &[&dyn Expression]) -> Result<&dyn Expression, ExpressionError> {
        Ok(self)
    }

    #[inline]
    fn equal(&self, other: &dyn Expression) -> bool {
        other
            .as_any()
            .downcast_ref::<Self>()
            .map_or(false, |e| self.name() == e.name())
    }

    #[inline]
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
