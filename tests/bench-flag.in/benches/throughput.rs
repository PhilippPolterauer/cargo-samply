fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let sum = bench_fixture::add(&[1, 2, 3, 4]);
    if args.is_empty() {
        println!("criterion bench sum={sum}");
    } else {
        println!("criterion bench sum={sum} args={}", args.join(" "));
    }
}
