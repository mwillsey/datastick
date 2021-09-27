use lalrpop_util::lalrpop_mod;

lalrpop_mod!(
    #[allow(dead_code)]
    #[allow(warnings)]
    grammar
);

pub use grammar::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::Symbol;
    #[test]
    fn parse_ident() {
        let p = IdentParser::new();
        assert_eq!(p.parse("_foo_123").unwrap(), Symbol::new("_foo_123"));
        assert_eq!(p.parse("_").unwrap(), Symbol::new("_"));
        assert!(p.parse("0").is_err());
    }
}
