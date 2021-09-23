use std::convert::TryInto;

use super::*;

macro_rules! query {
    ( $( $sym:ident ($($val:tt),*) ),+ ) => {
        Query {
            atoms: vec![$(
                Atom {
                    relation: $sym,
                    terms: vec![$( value!($val)),*],
                }
            ),+]
        }
    };
}

macro_rules! value {
    ($ident:ident) => {
        Term::Variable($ident)
    };
    ($val:expr) => {
        Term::Value(Value($val))
    };
}

macro_rules! symbols {
    ($($id:ident),+) => {
        #[allow(non_snake_case, unused_variables)]
        let ($($id),+) = ($(Symbol::new(stringify!($id))),+);
    };
}

impl Database {
    fn eval_and_check<V, const N: usize>(
        &self,
        handle: QueryHandle,
        vars: &[Symbol],
        expected: &[[V; N]],
    ) where
        V: Type + Clone,
    {
        let idxs = self.get_indexes(handle, vars);
        let mut results = HashSet::<[Value; N]>::default();
        self.eval_query(handle, |tuple| {
            let vec: Vec<Value> = idxs.iter().map(|&i| tuple[i]).collect();
            results.insert(vec.try_into().unwrap());
        });

        let expected: HashSet<[Value; N]> = expected
            .iter()
            .map(|tuple| tuple.clone().map(|v| v.to_value()))
            .collect();

        assert_eq!(results, expected)
    }
}

#[test]
fn triangle() {
    symbols!(R, a, b, c);
    let mut db = Database::default();
    let n = 10;

    let mut tuples = vec![[0, 1], [1, 2], [2, 0]];
    for i in 0..n {
        for j in 0..n {
            if i < j {
                tuples.push([i, j])
            }
        }
    }

    db.add_relation(R, 2).insert_arrays(&tuples);
    let q1 = db.add_query(query!(R(a, b), R(b, c), R(c, a)));
    db.eval_and_check(q1, &[a, b, c], &[[0, 1, 2], [1, 2, 0], [2, 0, 1]]);
}
