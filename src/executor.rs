use std::sync::Arc;

use runtime::error::RuntimeError;
use runtime::{CoreId, Runtime};
use snafu::{ResultExt, Snafu};

use crate::catalog::CatalogList;
use crate::expression::Expression;

#[derive(Snafu, Debug)]
pub enum ExecutorError {
    #[snafu(display("executor runtime initialization failed: {source}"))]
    Runtime { source: RuntimeError },
}

#[derive(Debug)]
pub struct Executor {
    runtime: Runtime,
    catalog_list: Arc<CatalogList>,
}

impl Executor {
    pub fn new(require_cores: &[CoreId], catalog_list: Arc<CatalogList>) -> Result<Self, ExecutorError> {
        let mut runtime = Runtime::new(require_cores).context(RuntimeSnafu {})?;
        runtime.run();
        Ok(Self { runtime, catalog_list })
    }

    pub fn execute(&self, _expr: Box<dyn Expression>) {
        // self.runtime.run(f);
        todo!()
    }
}
