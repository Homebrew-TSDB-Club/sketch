use std::future::Future;

use crate::column::MutableChunk;
use crate::context::Context;

use super::error::ExprError;
use super::{ExprImplRef, ExprType, Expression};

impl Expression for MutableChunk {
    type EvalFut<'a> = impl Future<Output = Result<ExprImplRef<'a>, ExprError>>;

    fn evaluate(&self, _: &mut Context, _: &[ExprImplRef<'_>]) -> Self::EvalFut<'_> {
        async { Ok(self.as_impl_ref()) }
    }

    fn as_impl_ref(&self) -> ExprImplRef<'_> {
        ExprImplRef {
            expr_type: ExprType::Chunk,
            data: self,
        }
    }
}
