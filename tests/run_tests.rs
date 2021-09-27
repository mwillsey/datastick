use datastick::DatalogContext;

#[test]
fn test_passing() {
    let dir = std::fs::read_dir("tests/passing/").unwrap();
    for entry in dir {
        let entry = entry.unwrap();
        if entry.file_type().unwrap().is_file() {
            let path = entry.path();
            if path.ends_with(".dl") {
                let s = std::fs::read_to_string(&path).unwrap();
                let mut ctx = DatalogContext::default();
                println!("Running {:?}", path.file_name().unwrap());
                ctx.parse_and_eval(&s);
                ctx.run();
            }
        }
    }
}
