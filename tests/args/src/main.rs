fn main() {
    let args = std::env::args();

    let args_without_binary_path: Vec<String> = args.skip(1).collect();

    println!("{args_without_binary_path:#?}",);
    assert!(args_without_binary_path.is_empty());
}
