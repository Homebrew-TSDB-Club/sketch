use std::future::Future;
use std::sync::Arc;

use crate::context::Context;
use crate::source::Table;

use super::error::ExprError;
use super::{ExprImplRef, ExprType, Expression};

impl Expression for Arc<Table> {
    type EvalFut<'a> = impl Future<Output = Result<ExprImplRef<'a>, ExprError>>;

    fn evaluate(&self, _: &mut Context, _: &[ExprImplRef<'_>]) -> Self::EvalFut<'_> {
        async { Ok(self.as_impl_ref()) }
    }

    fn as_impl_ref(&self) -> ExprImplRef<'_> {
        ExprImplRef {
            expr_type: ExprType::Table,
            data: self,
        }
    }
}
