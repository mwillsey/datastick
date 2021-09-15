mod gj;

use crate::util::*;
pub use gj::CompiledQuery;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Value(u64);

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
pub struct Symbol(u64);

impl std::fmt::Display for Symbol {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

type Variable = Symbol;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum Term {
    Variable(Variable),
    Value(Value),
}

#[derive(Debug, Clone)]
struct Atom {
    relation: Symbol,
    terms: Vec<Term>,
}

impl Atom {
    fn vars(&self) -> impl Iterator<Item = Variable> + '_ {
        self.terms.iter().filter_map(|t| match t {
            Term::Variable(v) => Some(*v),
            _ => None,
        })
    }

    fn has_var(&self, v: Variable) -> bool {
        self.terms.contains(&Term::Variable(v))
    }
}

// #[derive(Clone)]
// pub enum Type {}

#[derive(Clone)]
pub struct Relation {
    set: IndexSet<Vec<Value>>,
    arity: usize,
    // schema: Vec<Type>,
}

impl Relation {
    pub fn insert(&mut self, tuple: &[Value]) {
        assert_eq!(tuple.len(), self.arity);
        self.set.insert(tuple.to_vec());
    }
}

#[derive(Default)]
pub struct Database {
    relations: IndexMap<Symbol, Relation>,
    queries: IndexMap<QueryHandle, CompiledQuery>,
    query_id: usize,
}

#[derive(Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Clone, Copy)]
pub struct QueryHandle(usize);

#[derive(Debug, Clone)]
pub struct Query {
    atoms: Vec<Atom>,
}

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
        self.queries.insert(handle, cq).unwrap();
        handle
    }

    pub fn eval_query<F>(&self, handle: QueryHandle, f: F)
    where
        F: FnMut(&[Value]),
    {
        let query = &self.queries[&handle];
        query.eval(self, f)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
