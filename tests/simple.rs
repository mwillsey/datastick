use std::any::TypeId;

use datastick::{ast::*, symbols, util::*, DatalogContext};

fn mkrel(symbol: Symbol, arity: usize) -> Relation {
    let schema = Schema::from_types(&vec![TypeId::of::<i32>(); arity]);
    Relation { symbol, schema }
}

fn var(symbol: Symbol) -> Term {
    Term::Variable(symbol)
}

#[test]
fn simple() {
    let mut ctx = DatalogContext::default();
    symbols!(edge, reachable, a, b, c);

    let mut edges = vec![];
    for i in [0, 1, 2, 3, 9] {
        edges.push(i.to_value());
        edges.push((i + 1).to_value());
    }

    ctx.add_relation(mkrel(edge, 2));
    ctx.add_relation(mkrel(reachable, 2));
    ctx.insert_many(edge, &edges);

    ctx.add_rule(Rule {
        head: vec![Atom {
            relation: reachable,
            terms: vec![var(a), var(b)],
        }],
        body: Query {
            atoms: vec![Atom {
                relation: edge,
                terms: vec![var(a), var(b)],
            }],
        },
    });

    ctx.add_rule(Rule {
        head: vec![Atom {
            relation: reachable,
            terms: vec![var(a), var(c)],
        }],
        body: Query {
            atoms: vec![
                Atom {
                    relation: edge,
                    terms: vec![var(a), var(b)],
                },
                Atom {
                    relation: reachable,
                    terms: vec![var(b), var(c)],
                },
            ],
        },
    });

    ctx.run();

    let mut actual = ctx.collect::<2>(reachable);
    println!("{:?}", actual);

    let mut expected = vec![[9.to_value(), 10.to_value()]];
    for i in 0..5 {
        for j in 0..5 {
            if i < j {
                expected.push([i.to_value(), j.to_value()])
            }
        }
    }

    actual.sort();
    actual.dedup();
    expected.sort();
    expected.dedup();
    assert_eq!(expected, actual)
}
