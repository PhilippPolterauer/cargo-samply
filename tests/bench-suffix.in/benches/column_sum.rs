fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let sum = bench_suffix_fixture::column_sum(&[1, 2, 3]);
    if args.is_empty() {
        println!("column_sum_bench result={sum}");
    } else {
        println!("column_sum_bench result={sum} args={}", args.join(" "));
    }
}
