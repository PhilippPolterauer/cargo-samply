use criterion::Criterion;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    println!("criterion fixture args={}", args.join(" "));

    let criterion = Criterion::default();
    // Parse Criterion's CLI flags to ensure we exercise the real argument layer.
    let _ = criterion.configure_from_args();
    // Exit early to keep the fixture output deterministic for snapshot tests.
    std::process::exit(0);
}
