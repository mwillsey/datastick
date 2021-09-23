use std::convert::TryInto;

use ast::*;
use db::QueryHandle;
use util::Symbol;

pub mod ast;
pub mod db;
pub mod util;

#[derive(Default)]
pub struct DatalogContext {
    db: db::Database,
    rules: Vec<(Rule, QueryHandle)>,
}

impl DatalogContext {
    pub fn add_rule(&mut self, rule: Rule) {
        let handle = self.db.add_query(rule.body.clone());
        self.rules.push((rule, handle));
    }

    pub fn add_fact(&mut self, fact: &Fact) {
        let rel = &mut self.db.relations[&fact.symbol];
        rel.insert(fact.args.as_slice())
    }

    pub fn add_relation(&mut self, relation: Relation) {
        let symbol = relation.symbol;
        let arity = relation.schema.len();
        self.db
            .relations
            .entry(symbol)
            .and_modify(|_| panic!("a relation was already here"))
            .or_insert(db::Relation::new(arity));
    }

    pub fn insert_many(&mut self, relation: Symbol, tuples: &[Value]) {
        let rel = self.db.relations.get_mut(&relation).unwrap();
        rel.insert_many(tuples)
    }

    pub fn for_each(&self, relation: Symbol, mut f: impl FnMut(&[Value])) {
        let rel = self.db.relations.get(&relation).unwrap();
        rel.set.iter().for_each(|tuple| f(tuple.as_ref()))
    }

    pub fn collect<const N: usize>(&self, relation: Symbol) -> Vec<[Value; N]> {
        let mut vec = vec![];
        self.for_each(relation, |tup| vec.push(tup.try_into().unwrap()));
        vec
    }

    pub fn run(&mut self) -> usize {
        let mut additions = 0;
        loop {
            let new = self.step();
            if new == 0 {
                return additions;
            }
            additions += new;
        }
    }

    pub fn step(&mut self) -> usize {
        let all_substs: Vec<Vec<_>> = self
            .rules
            .iter()
            .map(|(_r, qh)| {
                let mut vec = Vec::new();
                self.db.eval_query(*qh, |vals| vec.extend_from_slice(vals));
                vec
            })
            .collect();

        let mut additions = 0;
        for ((rule, handle), substs) in self.rules.iter().zip(all_substs) {
            // get the vars, we only handle vars (no expressions) for now
            assert_eq!(rule.head.len(), 1);
            let atom = &rule.head[0];
            let mut vars = Vec::with_capacity(atom.terms.len());
            for term in &atom.terms {
                match term {
                    Term::Variable(v) => vars.push(*v),
                    Term::Value(_) => panic!(),
                }
            }

            let idxs = self.db.get_indexes(*handle, &vars);
            let subst_len = self.db.get_subst_len(*handle);

            let rel = self.db.relations.get_mut(&atom.relation).unwrap();
            let initial_size = rel.len();

            println!("{:?}", vars);
            println!("{:?}", idxs);

            let mut tuple = vec![Value::default(); rel.arity];

            for subst in substs.chunks_exact(subst_len) {
                for (i, idx) in idxs.iter().enumerate() {
                    tuple[i] = subst[*idx];
                }
                rel.insert(&tuple)
            }

            additions += rel.len() - initial_size;
        }

        additions
    }
}
