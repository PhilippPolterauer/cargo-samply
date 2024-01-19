fn main(){
    // read arguments
    let args = std::env::args().collect::<Vec<String>>();
    if args.len() < 2 {
        println!("Hello, world!");
    }else{
        println!("Hello, {}!", args[1]);
    }
}
