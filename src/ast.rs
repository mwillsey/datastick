use crate::util::Symbol;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Value(u64);

pub struct Rule {
    pub head: Vec<Atom>,
    pub body: Vec<Atom>,
}

pub type Variable = Symbol;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum Term {
    Variable(Variable),
    Value(Value),
}

#[derive(Debug, Clone)]
pub struct Atom {
    pub relation: Symbol,
    pub terms: Vec<Term>,
}

impl Atom {
    pub fn vars(&self) -> impl Iterator<Item = Variable> + '_ {
        self.terms.iter().filter_map(|t| match t {
            Term::Variable(v) => Some(*v),
            _ => None,
        })
    }

    pub fn has_var(&self, v: Variable) -> bool {
        self.terms.contains(&Term::Variable(v))
    }
}
