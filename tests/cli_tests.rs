use regex::Regex;
use which::{which, which_re};

#[test]
fn trycmd() {
    let cargo_bins = which_re(Regex::new("^cargo-.*").unwrap())
        .unwrap()
        .collect::<Vec<_>>();
    let test = trycmd::TestCases::new();
    let mut t = test
        // .case("README.md")
        .case("tests/*.trycmd")
        .register_bin("cargo", trycmd::schema::Bin::Path(which("cargo").unwrap()));

    for pth in cargo_bins.iter().filter(|pth| {
        !pth.extension().is_some_and(|pth| pth == "exe") && pth.file_name().is_some_and(|p| p !="cargo-samply")
    }) {
        println!("{}", pth.file_name().unwrap().to_string_lossy());
        t = t.register_bin(
            pth.file_name().unwrap().to_string_lossy(),
            trycmd::schema::Bin::Path(pth.clone()),
        );
    }
    t.register_bin("cargo-samply", trycmd::cargo::cargo_bin("cargo-samply"));
}
