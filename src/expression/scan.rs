use std::sync::Arc;

use crate::catalog::CatalogList;
use crate::context::Context;

use super::error::ExpressionError;
use super::{Expression, Literal};

#[derive(Debug)]
pub struct Scanner {
    catalog_list: Arc<CatalogList>,
    output: Box<dyn Expression>,
}

impl Scanner {
    pub fn new(catalog_list: Arc<CatalogList>, output: Box<dyn Expression>) -> Self {
        Self { catalog_list, output }
    }
}

impl Expression for Scanner {
    fn evaluate(&self, context: &mut Context, args: &[&dyn Expression]) -> Result<&dyn Expression, ExpressionError> {
        let literal = args[0]
            .evaluate(context, &[])?
            .as_any()
            .downcast_ref::<Literal>()
            .unwrap();
        let resource = literal.as_ref().rsplit('.').collect::<Vec<_>>();
        if resource.len() < 1 {
            return Err(ExpressionError::InvalidResource {
                resource: literal.clone(),
            });
        }
        let (catalog, schema, table) = (resource.get(2).cloned(), resource.get(1).cloned(), resource[0]);
        let catalog = match catalog {
            Some(catalog) => self
                .catalog_list
                .get(catalog)
                .ok_or_else(|| ExpressionError::ResourceNotFound {
                    resource: catalog.to_owned(),
                })?,
            None => self.catalog_list.get_default(),
        };
        let schema = match schema {
            Some(schema) => catalog.get(schema).ok_or_else(|| ExpressionError::ResourceNotFound {
                resource: schema.to_owned(),
            })?,
            None => catalog.get_default(),
        };
        let table = schema.get(table).ok_or_else(|| ExpressionError::ResourceNotFound {
            resource: table.to_owned(),
        })?;

        self.output.evaluate(context, &[table.as_ref()])
    }

    fn equal(&self, _other: &dyn Expression) -> bool {
        true
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

// #[cfg(test)]
// mod tests {
//     use std::sync::Arc;

//     use crate::catalog::CatalogList;
//     use crate::context::Context;
//     use crate::expression::Literal;

//     use super::Scanner;

//     #[test]
//     fn test_scan() {
//         let catalog_list = CatalogList::new();
//         let scanner = Scanner {
//             catalog_list: Arc::new(catalog_list),
//             output: Box::new(Literal(String::from("hello"))),
//         };
//         scanner.evaluate(&mut Context::new(), &[&Literal(String::from("hello"))]);
//     }
// }
