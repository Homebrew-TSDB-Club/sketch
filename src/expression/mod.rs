// pub mod chunk;
pub mod error;
// pub mod filter;
// pub mod scan;
// pub mod source;

use std::any::Any;
use std::fmt::Debug;
use std::future::Future;

use crate::context::Context;
use crate::primitive::{Primitive as PrimitiveData, PrimitiveType};

use self::error::ExprError;

#[derive(Debug, PartialEq, Clone)]
pub enum ExprType {
    String,
    Literal,
    Primitive(PrimitiveType),
}

pub trait AnyExpr: Any + Debug {
    fn clone_box(&self) -> Box<dyn AnyExpr>;
    fn equal(&self, other: &dyn AnyExpr) -> bool;
    fn as_any(&self) -> &dyn Any;
}

impl<T: Expression> AnyExpr for T {
    #[inline]
    fn clone_box(&self) -> Box<dyn AnyExpr> {
        Box::new(self.clone())
    }

    fn equal(&self, other: &dyn AnyExpr) -> bool {
        other.as_any().downcast_ref::<Self>().map_or(false, |e| self == e)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl PartialEq for &dyn AnyExpr {
    fn eq(&self, other: &Self) -> bool {
        self.equal(*other)
    }
}

#[derive(Debug)]
pub struct ExprImpl {
    expr_type: ExprType,
    data: Box<dyn AnyExpr>,
}

impl PartialEq for ExprImpl {
    fn eq(&self, other: &Self) -> bool {
        self.expr_type == other.expr_type && &*self.data == &*other.data
    }
}

#[derive(Debug, PartialEq)]
pub struct ExprImplRef<'a> {
    expr_type: ExprType,
    data: &'a dyn AnyExpr,
}

impl ExprImplRef<'_> {
    pub fn to_owned(&self) -> ExprImpl {
        ExprImpl {
            expr_type: self.expr_type.clone(),
            data: self.data.clone_box(),
        }
    }
}

pub trait Expression: 'static + Send + Debug + Clone + PartialEq {
    type EvalFut<'a>: Future<Output = Result<ExprImplRef<'a>, ExprError>>
    where
        Self: 'a;

    fn evaluate(&self, context: &mut Context, args: &[ExprImplRef<'_>]) -> Self::EvalFut<'_>;
    fn as_impl_ref(&self) -> ExprImplRef<'_>;
}

impl Expression for String {
    type EvalFut<'a> = impl Future<Output = Result<ExprImplRef<'a>, ExprError>>;

    #[inline]
    fn evaluate(&self, _context: &mut Context, _args: &[ExprImplRef<'_>]) -> Self::EvalFut<'_> {
        async { Ok(self.as_impl_ref()) }
    }

    #[inline]
    fn as_impl_ref(&self) -> ExprImplRef<'_> {
        ExprImplRef {
            expr_type: ExprType::String,
            data: self,
        }
    }
}

#[repr(transparent)]
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Primitive<T: PrimitiveData>(T);

impl<T: PrimitiveData> Expression for Primitive<T> {
    type EvalFut<'a> = impl Future<Output = Result<ExprImplRef<'a>, ExprError>>;

    #[inline]
    fn evaluate(&self, _context: &mut Context, _args: &[ExprImplRef<'_>]) -> Self::EvalFut<'_> {
        async { Ok(self.as_impl_ref()) }
    }

    #[inline]
    fn as_impl_ref(&self) -> ExprImplRef<'_> {
        ExprImplRef {
            expr_type: ExprType::Primitive(T::TYPE),
            data: self,
        }
    }
}

#[repr(transparent)]
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Literal(String);

impl AsRef<str> for Literal {
    #[inline]
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl Expression for Literal {
    type EvalFut<'a> = impl Future<Output = Result<ExprImplRef<'a>, ExprError>>;

    #[inline]
    fn evaluate(&self, _context: &mut Context, _args: &[ExprImplRef<'_>]) -> Self::EvalFut<'_> {
        async { Ok(self.as_impl_ref()) }
    }

    #[inline]
    fn as_impl_ref(&self) -> ExprImplRef<'_> {
        ExprImplRef {
            expr_type: ExprType::Literal,
            data: self,
        }
    }
}

// sum by (job) (http_requests_total{job="prometheus"}[5m])
// SELECT SUM(value) FROM http_requests_total WHERE job = "prometheus" AND (now - 5m) < time < now GROUP BY job

// FROM http_requests_total
// WHERE job = "prometheus"
// RANGE (now - 5m) < time < now
// SUM(value) BY job
