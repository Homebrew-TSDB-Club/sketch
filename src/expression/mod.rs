pub mod chunk;
pub mod error;
pub mod filter;
pub mod scan;
pub mod source;

use std::any::Any;
use std::fmt::Debug;

use crate::primitive::Primitive;

use self::error::ExpressionError;

pub trait Expression: 'static + Send + Debug {
    fn evaluate(
        &self,
        context: &mut crate::context::Context,
        args: &[&dyn Expression],
    ) -> Result<&dyn Expression, ExpressionError>;
    fn equal(&self, other: &dyn Expression) -> bool;
    fn as_any(&self) -> &dyn Any;

    #[inline]
    fn boxed(self) -> Box<dyn Expression>
    where
        Self: Sized,
    {
        Box::new(self)
    }
}

impl PartialEq for dyn Expression {
    fn eq(&self, other: &Self) -> bool {
        self.equal(other)
    }
}

impl Expression for String {
    #[inline]
    fn evaluate(
        &self,
        _context: &mut crate::context::Context,
        _args: &[&dyn Expression],
    ) -> Result<&dyn Expression, ExpressionError> {
        Ok(self)
    }

    #[inline]
    fn equal(&self, other: &dyn Expression) -> bool {
        other.as_any().downcast_ref::<Self>().map_or(false, |e| self == e)
    }

    #[inline]
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[repr(transparent)]
#[derive(Debug, PartialEq, Eq)]
pub struct PrimitiveExpr<T: Primitive>(T);

impl<T: Primitive> Expression for PrimitiveExpr<T> {
    #[inline]
    fn evaluate(
        &self,
        _context: &mut crate::context::Context,
        _args: &[&dyn Expression],
    ) -> Result<&dyn Expression, ExpressionError> {
        Ok(self)
    }

    #[inline]
    fn equal(&self, other: &dyn Expression) -> bool {
        other.as_any().downcast_ref::<Self>().map_or(false, |e| self == e)
    }

    #[inline]
    fn as_any(&self) -> &dyn Any {
        self
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
    #[inline]
    fn evaluate(
        &self,
        _context: &mut crate::context::Context,
        _args: &[&dyn Expression],
    ) -> Result<&dyn Expression, ExpressionError> {
        Ok(self)
    }

    #[inline]
    fn equal(&self, other: &dyn Expression) -> bool {
        other.as_any().downcast_ref::<Self>().map_or(false, |e| self == e)
    }

    #[inline]
    fn as_any(&self) -> &dyn Any {
        self
    }
}

// sum by (job) (http_requests_total{job="prometheus"}[5m])
// SELECT SUM(value) FROM http_requests_total WHERE job = "prometheus" AND (now - 5m) < time < now GROUP BY job

// FROM http_requests_total
// WHERE job = "prometheus"
// RANGE (now - 5m) < time < now
// SUM(value) BY job
