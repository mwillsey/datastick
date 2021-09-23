mod gj;

#[cfg(test)]
mod tests;

use crate::ast::*;
use crate::util::*;

pub use gj::CompiledQuery;

#[derive(Clone)]
pub struct Relation {
    set: IndexSet<Vec<Value>>,
    arity: usize,
    // schema: Vec<Type>,
}

impl Relation {
    pub fn new(arity: usize) -> Relation {
        Self {
            set: Default::default(),
            arity,
        }
    }

    pub fn insert(&mut self, tuple: &[Value]) {
        assert_eq!(tuple.len(), self.arity);
        self.set.insert(tuple.to_vec());
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

    pub fn get_relation(&self, symbol: Symbol) -> &Relation {
        self.relations
            .get(&symbol)
            .unwrap_or_else(|| panic!("Relation {} does not exist", symbol))
    }

    pub fn get_mut_relation(&mut self, symbol: Symbol) -> &mut Relation {
        self.relations
            .get_mut(&symbol)
            .unwrap_or_else(|| panic!("Relation {} does not exist", symbol))
    }

    pub fn remove_relation(&mut self, symbol: Symbol) -> Relation {
        self.relations
            .remove(&symbol)
            .unwrap_or_else(|| panic!("Relation {} does not exist", symbol))
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

    pub fn get_index(&self, handle: QueryHandle, var: Symbol) -> usize {
        self.queries[&handle].get_index(var)
    }

    pub fn get_indexes(&self, handle: QueryHandle, vars: &[Symbol]) -> Vec<usize> {
        let q = &self.queries[&handle];
        vars.iter().map(|&v| q.get_index(v)).collect()
    }

    pub fn collect(&self, handle: QueryHandle) -> Vec<Value> {
        let mut vec = vec![];
        self.collect_into(handle, &mut vec);
        vec
    }

    pub fn collect_into(&self, handle: QueryHandle, vec: &mut Vec<Value>) {
        let query = &self.queries[&handle];
        query.eval(self, |values| vec.extend_from_slice(values));
    }
}
