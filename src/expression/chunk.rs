use crate::column::MutableChunk;

use super::error::ExpressionError;
use super::Expression;

impl Expression for MutableChunk {
    fn evaluate(
        &self,
        _context: &mut crate::context::Context,
        _args: &[&dyn Expression],
    ) -> Result<&dyn Expression, ExpressionError> {
        Ok(self)
    }

    #[inline]
    fn equal(&self, _other: &dyn Expression) -> bool {
        unimplemented!()
    }

    #[inline]
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
