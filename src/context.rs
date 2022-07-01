use std::sync::Arc;

use hashbrown::HashMap;

use crate::expression::Expression;

#[derive(Debug, Default)]
pub(crate) struct Stack {
    variables: HashMap<Arc<str>, Box<dyn Expression>>,
}

impl Stack {
    fn new() -> Self {
        Default::default()
    }

    fn get(&self, name: &str) -> Option<&Box<dyn Expression>> {
        self.variables.get(name)
    }

    fn get_mut(&mut self, name: &str) -> Option<&mut Box<dyn Expression>> {
        self.variables.get_mut(name)
    }

    fn set(&mut self, name: &str, value: Box<dyn Expression>) {
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

    pub fn get(&self, name: &str) -> Option<&Box<dyn Expression>> {
        self.stack.iter().rev().find_map(|c| c.get(name))
    }

    pub fn set(&mut self, name: &str, value: Box<dyn Expression>) {
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
            env.set("foo", String::from("foo").boxed());
            env.push(|env| {
                assert!(env.get("foo") == Some(&String::from("foo").boxed()));
                env.set("bar", String::from("bar").boxed());
                assert!(env.get("bar") == Some(&String::from("bar").boxed()));
                env.set("foo", String::from("baz").boxed());
                assert!(env.get("foo") == Some(&String::from("baz").boxed()));
            });
            assert!(env.get("bar") == None);
            assert!(env.get("foo") == Some(&String::from("baz").boxed()));
        });
    }
}
