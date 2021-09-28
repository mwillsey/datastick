use datastick::DatalogContext;

fn tests_in_dir(dir: &str) -> impl Iterator<Item = String> {
    std::fs::read_dir(dir)
        .unwrap()
        .into_iter()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.is_file() && path.extension().unwrap_or_default() == "dl")
        .inspect(|path| println!("Test {:?}", path))
        .map(|path| std::fs::read_to_string(&path).unwrap())
}

#[test]
fn test_passing() {
    for s in tests_in_dir("tests/pass") {
        let mut ctx = DatalogContext::default();
        ctx.parse_and_eval(&s)
    }
}
