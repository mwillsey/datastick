use std::any::{Any, TypeId};

use crate::util::{HashSet, Symbol};

#[derive(Default, Debug, PartialEq, Eq, Hash, Clone, Copy, PartialOrd, Ord)]
pub struct Value(u64);

pub type Variable = Symbol;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum Term {
    Variable(Variable),
    Value(Value),
}

impl Term {
    pub fn eval(&self) -> Value {
        match self {
            Term::Variable(v) => panic!("Can't eval a variable {}", v),
            Term::Value(val) => *val,
        }
    }
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

#[derive(Debug, Clone)]
pub struct Rule {
    pub head: Vec<Atom>,
    pub body: Query,
}

#[derive(Debug, Clone)]
pub struct Query {
    pub atoms: Vec<Atom>,
}

#[derive(Debug, Clone)]
pub struct Relation {
    pub symbol: Symbol,
    pub schema: Schema,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Schema {
    types: Vec<(Symbol, TypeId)>,
}

impl Schema {
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn len(&self) -> usize {
        self.types.len()
    }

    pub fn from_types(types: &[TypeId]) -> Self {
        let types: Vec<_> = types
            .iter()
            .enumerate()
            .map(|(i, &t)| (Symbol::new(format!("{}", i)), t))
            .collect();
        Self { types }
    }

    pub fn from_named_types(types: Vec<(Symbol, TypeId)>) -> Self {
        let mut names = HashSet::default();
        for &(s, _t) in &types {
            let was_inserted = names.insert(s);
            assert!(was_inserted);
        }
        Self { types }
    }
}

#[macro_export]
macro_rules! schema {
    ($($s:ident : $t:ty),*) => {
        $crate::Schema::from_named_types(vec![$(
            ( $crate::ast::Symbol::new(stringify!($s)),
              <$t as $crate::Type>::type_id())
        ),*])
    };
    ($($t:ty),*) => {
        $crate::Schema::from_types(&[$(
            <$t as $crate::Type>::type_id()
        ),*])
    };
}

#[derive(Debug, Clone)]
pub enum Directive {
    AssertEq(Symbol, Symbol),
}

#[derive(Default, Debug, Clone)]
pub struct Program {
    pub rules: Vec<Rule>,
    pub relations: Vec<Relation>,
    pub facts: Vec<Atom>,
    pub directives: Vec<Directive>,
}

// TODO rename
pub trait Type: Any {
    fn type_id() -> TypeId {
        TypeId::of::<Self>()
    }
    fn to_value(self) -> Value;
}

impl Type for i32 {
    fn to_value(self) -> Value {
        Value(self as u64)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_schema() {
        let s1 = schema!(a: i32, b: i32);
        let s2 = schema!(a: i32, b: i32);
        assert_eq!(s1, s2);

        let s3 = schema!(b: i32, a: i32);
        let s4 = schema!(i32, i32);
        assert_ne!(s1, s3);
        assert_ne!(s1, s4);

        let s5 = schema!(i32, i32);
        assert_eq!(s4, s5);
    }
}
