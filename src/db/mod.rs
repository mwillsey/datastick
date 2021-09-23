mod gj;

#[cfg(test)]
mod tests;

use std::borrow::BorrowMut;

use crate::ast::*;
use crate::util::*;

pub use gj::CompiledQuery;

#[derive(Clone)]
pub struct Relation {
    // TODO shouldn't be pub
    pub set: IndexSet<Vec<Value>>,
    pub arity: usize,
    // schema: Vec<Type>,
}

impl Relation {
    pub fn new(arity: usize) -> Relation {
        Self {
            set: Default::default(),
            arity,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.set.is_empty()
    }

    pub fn len(&self) -> usize {
        let n = self.set.len();
        n.checked_div(self.arity).unwrap_or(n)
    }

    pub fn insert(&mut self, tuple: &[Value]) {
        assert_eq!(tuple.len(), self.arity);
        self.set.insert(tuple.to_vec());
    }

    pub fn insert_many(&mut self, tuples: &[Value]) {
        assert_eq!(tuples.len() % self.arity, 0);
        for tuple in tuples.chunks_exact(self.arity) {
            self.set.insert(tuple.to_vec());
        }
    }

    pub fn insert_arrays<V, const N: usize>(&mut self, tuples: &[[V; N]])
    where
        V: Clone + Type,
    {
        assert_eq!(self.arity, N);
        for tuple in tuples {
            let tuple = tuple.clone().map(|v| v.to_value());
            self.insert(&tuple);
        }
    }
}

#[derive(Default)]
pub struct Database {
    pub relations: IndexMap<Symbol, Relation>,
    queries: IndexMap<QueryHandle, CompiledQuery>,
    query_id: usize,
}

#[derive(Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Clone, Copy)]
pub struct QueryHandle(usize);

impl Database {
    pub fn add_relation(&mut self, symbol: Symbol, arity: usize) -> &mut Relation {
        self.relations
            .entry(symbol)
            .and_modify(|_| panic!("a relation was already here"))
            .or_insert(Relation {
                set: Default::default(),
                arity,
            })
    }
}

impl Database {
    pub fn add_query(&mut self, query: Query) -> QueryHandle {
        let cq = CompiledQuery::new(self, query);
        let handle = QueryHandle(self.query_id);
        self.query_id += 1;
        let old = self.queries.insert(handle, cq);
        assert!(old.is_none());
        handle
    }

    pub fn eval_query<F>(&self, handle: QueryHandle, f: F)
    where
        F: FnMut(&[Value]),
    {
        let query = &self.queries[&handle];
        query.eval(self, f)
    }

    pub fn get_indexes(&self, handle: QueryHandle, vars: &[Symbol]) -> Vec<usize> {
        let q = &self.queries[&handle];
        vars.iter().map(|&v| q.get_index(v)).collect()
    }

    // TODO this is an awful api
    pub fn get_subst_len(&self, handle: QueryHandle) -> usize {
        let q = &self.queries[&handle];
        q.by_var.len()
    }

    pub fn collect(&self, handle: QueryHandle) -> Vec<Value> {
        self.collect_into(handle, vec![])
    }

    pub fn collect_into<V>(&self, handle: QueryHandle, mut vec: V) -> V
    where
        V: BorrowMut<Vec<Value>>,
    {
        let v = vec.borrow_mut();
        let query = &self.queries[&handle];
        query.eval(self, |values| v.extend_from_slice(values));
        vec
    }
}
