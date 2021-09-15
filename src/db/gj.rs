use super::*;

use crate::util::IndexMap;

#[derive(Default, Debug, Clone)]
struct Trie(IndexMap<Value, Self>);

impl Trie {
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl Trie {
    fn insert(&mut self, shuffle: &[usize], tuple: &[Value]) {
        debug_assert_eq!(shuffle.len(), tuple.len());
        let mut trie = self;
        for i in shuffle {
            trie = trie.0.entry(tuple[*i].clone()).or_default();
        }
    }
}

// for each var, says which atoms contain it
type VarOccurences = IndexMap<Variable, Vec<usize>>;

#[derive(Clone)]
pub struct CompiledQuery {
    query: Query,
    by_var: VarOccurences,
}

impl CompiledQuery {
    pub fn new(_db: &Database, query: Query) -> Self {
        let mut by_var = VarOccurences::default();
        for (i, atom) in query.atoms.iter().enumerate() {
            for v in atom.vars() {
                let is = by_var.entry(v).or_default();
                if is.last().copied() != Some(i) {
                    is.push(i)
                }
            }
        }

        // simple variable ordering for now
        by_var.sort_by(|_v1, occ1, _v2, occ2| occ1.len().cmp(&occ2.len()).reverse());

        if cfg!(debug_assert) {
            for (&var, ats) in &by_var {
                let expected: Vec<usize> = query
                    .atoms
                    .iter()
                    .enumerate()
                    .filter_map(|(i, a)| a.has_var(var).then(|| i))
                    .collect();
                debug_assert_eq!(ats, &expected)
            }
        }

        Self { query, by_var }
    }

    pub fn eval<F>(&self, db: &Database, mut f: F)
    where
        F: FnMut(&[Value]),
    {
        let tries = self
            .query
            .atoms
            .iter()
            .map(|atom| {
                let mut shuffle = vec![];
                for var in self.by_var.keys() {
                    for (i, term) in atom.terms.iter().enumerate() {
                        if let Term::Variable(v) = term {
                            if var == v && !shuffle.contains(&i) {
                                shuffle.push(i);
                            }
                        }
                    }
                }

                let mut trie = Trie::default();
                for tuple in &db.relations[&atom.relation].set {
                    trie.insert(&shuffle, tuple);
                }

                trie
            })
            .collect::<Vec<_>>();

        let tries: Vec<&Trie> = tries.iter().collect();

        self.gj(&mut f, &[], &tries);
    }

    fn gj<F>(&self, f: &mut F, tuple: &[Value], relations: &[&Trie])
    where
        F: FnMut(&[Value]),
    {
        // println!("{:?}", tuple);
        if tuple.len() == self.by_var.len() {
            return f(tuple);
        }

        assert!(tuple.len() < self.by_var.len());

        let (&x, js) = self.by_var.get_index(tuple.len()).unwrap();
        debug_assert!(js.iter().all(|&j| self.query.atoms[j].has_var(x)));

        let j_min = js
            .iter()
            .copied()
            .min_by_key(|j| relations[*j].len())
            .unwrap();

        // for &j in js {
        //     println!("{:?}", relations[j].0.keys());
        // }

        let mut intersection: Vec<Value> = relations[j_min].0.keys().cloned().collect();

        for &j in js {
            if j != j_min {
                let rj = &relations[j].0;
                intersection.retain(|t| rj.contains_key(t));
            }
        }

        // println!("intersection of {:?}: {:?}", x, intersection);

        let empty = Trie::default();

        let mut tuple = tuple.to_vec();
        for val in intersection {
            let relations: Vec<_> = relations
                .iter()
                .zip(&self.query.atoms)
                .map(|(r, a)| {
                    if a.has_var(x) {
                        r.0.get(&val).unwrap_or(&empty)
                    } else {
                        r
                    }
                })
                .collect();
            tuple.push(val);
            self.gj(f, &tuple, &relations);
            tuple.pop();
        }
    }
}
