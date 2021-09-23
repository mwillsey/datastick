use ast::*;
use db::QueryHandle;

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

    pub fn step(&mut self) {
        let _all_substs: Vec<Vec<_>> = self
            .rules
            .iter()
            .map(|(_r, qh)| {
                let mut vec = Vec::new();
                self.db.eval_query(*qh, |vals| vec.extend_from_slice(vals));
                vec
            })
            .collect();
        todo!()
    }
}
