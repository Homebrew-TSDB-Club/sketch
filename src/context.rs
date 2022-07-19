use std::sync::Arc;

use hashbrown::HashMap;

use crate::expression::ExprImpl;

#[derive(Debug, Default)]
pub(crate) struct Stack {
    variables: HashMap<Arc<str>, Box<ExprImpl>>,
}

impl Stack {
    fn new() -> Self {
        Default::default()
    }

    fn get(&self, name: &str) -> Option<&Box<ExprImpl>> {
        self.variables.get(name)
    }

    fn get_mut(&mut self, name: &str) -> Option<&mut Box<ExprImpl>> {
        self.variables.get_mut(name)
    }

    fn set(&mut self, name: &str, value: Box<ExprImpl>) {
        self.variables.insert(Arc::from(name), value);
    }
}

#[derive(Debug, Default)]
pub struct Context {
    stack: Vec<Stack>,
}

impl Context {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn push(&mut self, f: impl FnOnce(&mut Self)) {
        self.stack.push(Stack::new());
        f(self);
        self.stack.pop();
    }

    pub fn get(&self, name: &str) -> Option<&Box<ExprImpl>> {
        self.stack.iter().rev().find_map(|c| c.get(name))
    }

    pub fn set(&mut self, name: &str, value: Box<ExprImpl>) {
        for context in self.stack.iter_mut().rev() {
            if let Some(v) = context.get_mut(name) {
                *v = value;
                return;
            }
        }
        self.stack.last_mut().unwrap().set(name, value.into());
    }
}

#[cfg(test)]
mod tests {
    use crate::expression::Expression;

    use super::Context;

    #[test]
    fn test_context() {
        let mut env = Context::new();
        env.push(|env| {
            env.set("foo", Box::new(String::from("foo").as_impl_ref().to_owned()));
            env.push(|env| {
                assert!(env.get("foo") == Some(&Box::new(String::from("foo").as_impl_ref().to_owned())));
                env.set("bar", Box::new(String::from("bar").as_impl_ref().to_owned()));
                assert!(env.get("bar") == Some(&Box::new(String::from("bar").as_impl_ref().to_owned())));
                env.set("foo", Box::new(String::from("baz").as_impl_ref().to_owned()));
                assert!(env.get("foo") == Some(&Box::new(String::from("baz").as_impl_ref().to_owned())));
            });
            assert!(env.get("bar") == None);
            assert!(env.get("foo") == Some(&Box::new(String::from("baz").as_impl_ref().to_owned())));
        });
    }
}
