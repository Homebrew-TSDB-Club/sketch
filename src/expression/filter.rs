use super::error::ExpressionError;
use super::Expression;

#[derive(Debug)]
pub struct Filter {
    predicate: Box<dyn Expression>,
}

#[derive(Debug, PartialEq)]
pub enum BinaryOp {
    ExactMatch,
    ExactNotMatch,
    RegexMatch,
    RegexNotMatch,
    And,
    Or,
}

#[derive(Debug)]
pub struct Predicate {
    op: BinaryOp,
    lhs: Box<dyn Expression>,
    rhs: Box<dyn Expression>,
}

impl Expression for Predicate {
    #[inline]
    fn evaluate(
        &self,
        context: &mut crate::context::Context,
        args: &[&dyn Expression],
    ) -> Result<&dyn Expression, ExpressionError> {
        let array = args[0].evaluate(context, &[]);
        todo!()
    }

    #[inline]
    fn equal(&self, other: &dyn Expression) -> bool {
        other.as_any().downcast_ref::<Self>().map_or(false, |e| self == e)
    }

    #[inline]
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl PartialEq for Predicate {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        &other.op == &self.op && &other.lhs == &self.lhs && &other.rhs == &self.rhs
    }
}

#[derive(Debug, PartialEq)]
pub struct Column {
    name: String,
}

impl Expression for Column {
    #[inline]
    fn evaluate(
        &self,
        _context: &mut crate::context::Context,
        _args: &[&dyn Expression],
    ) -> Result<&dyn Expression, ExpressionError> {
        todo!()
    }

    #[inline]
    fn equal(&self, other: &dyn Expression) -> bool {
        other.as_any().downcast_ref::<Self>().map_or(false, |e| self == e)
    }

    #[inline]
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::{BinaryOp, Column, Filter, Predicate};

    #[test]
    fn express_filter() {
        let predicate_1 = Box::new(Predicate {
            op: BinaryOp::ExactMatch,
            lhs: Box::new(Column {
                name: String::from("job"),
            }),
            rhs: Box::new(String::from("prometheus")),
        });
        let predicate_2 = Box::new(Predicate {
            op: BinaryOp::ExactMatch,
            lhs: Box::new(Column {
                name: String::from("foo"),
            }),
            rhs: Box::new(String::from("bar")),
        });
        let predicate = Box::new(Predicate {
            op: BinaryOp::And,
            lhs: predicate_1,
            rhs: predicate_2,
        });
        let filter = Filter { predicate };
        println!("{:?}", filter);
    }
}
