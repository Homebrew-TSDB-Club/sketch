use std::future::Future;
use std::sync::Arc;

use crate::catalog::CatalogList;
use crate::context::Context;
use crate::source::Table;

use super::error::ExprError;
use super::{ExprImpl, ExprImplRef, ExprType, Expression, Literal};

#[derive(Debug, Clone)]
pub struct Scanner {
    catalog_list: Arc<CatalogList>,
    output: Box<ExprImpl>,
}

impl Scanner {
    pub fn new(catalog_list: Arc<CatalogList>, output: Box<ExprImpl>) -> Self {
        Self { catalog_list, output }
    }
}

impl Expression for Scanner {
    type EvalFut<'a> = impl Future<Output = Result<ExprImplRef<'a>, ExprError>> + 'a;

    fn evaluate<'a>(&'a self, context: &'a mut Context, args: &'a [ExprImplRef<'a>]) -> Self::EvalFut<'a> {
        async {
            let literal = args.get(0).unwrap().evaluate(context, &[]).await?;
            let literal = literal.as_any().downcast_ref::<Literal>().unwrap();
            let resource = literal.as_ref().rsplit('.').collect::<Vec<_>>();
            if resource.len() < 1 {
                return Err(ExprError::InvalidResource {
                    resource: literal.clone(),
                });
            }
            let (catalog, schema, table) = (resource.get(2).cloned(), resource.get(1).cloned(), resource[0]);
            let catalog = match catalog {
                Some(catalog) => self
                    .catalog_list
                    .get(catalog)
                    .ok_or_else(|| ExprError::ResourceNotFound {
                        resource: catalog.to_owned(),
                    })?,
                None => self.catalog_list.get_default(),
            };
            let schema = match schema {
                Some(schema) => catalog.get(schema).ok_or_else(|| ExprError::ResourceNotFound {
                    resource: schema.to_owned(),
                })?,
                None => catalog.get_default(),
            };
            let table = schema.get(table).ok_or_else(|| ExprError::ResourceNotFound {
                resource: table.to_owned(),
            })?;
            let args = vec![table.as_impl_ref()];
            let eval = self.output.evaluate(context, &args).await;
            todo!()
            // eval
        }
    }

    fn as_impl_ref(&self) -> super::ExprImplRef<'_> {
        todo!()
    }
}

impl PartialEq for Scanner {
    fn eq(&self, _other: &Self) -> bool {
        true
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
