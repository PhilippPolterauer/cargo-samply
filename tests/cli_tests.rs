use regex::Regex;
use which::{which, which_re};

#[test]
fn trycmd() {
    let cargo_bins = which_re(Regex::new("^cargo-.*").unwrap())
        .unwrap()
        .collect::<Vec<_>>();
    let test = trycmd::TestCases::new();
    let fake_samply = std::path::PathBuf::from(env!("CARGO_BIN_EXE_fake-samply"));
    let mut t = test
        .env("TERM", "dumb")
        .env("CARGO_TERM_QUIET", "true")
        .env(
            "CARGO_SAMPLY_SAMPLY_PATH",
            fake_samply.display().to_string(),
        );

    t.register_bin("fake-samply", fake_samply);

    // Load trycmd cases. On Windows we avoid loading cases known to be
    // problematic by not including the `tests/skip-on-windows` folder.
    if cfg!(windows) {
        t = t.case("tests/*.trycmd");
    } else {
        t = t
            .case("tests/*.trycmd")
            .case("tests/skip-on-windows/*.trycmd");
    }

    t = t.register_bin("cargo", trycmd::schema::Bin::Path(which("cargo").unwrap()));

    for pth in cargo_bins.iter().filter(|pth| {
        pth.extension().is_some_and(|pth| pth == "exe")
            && pth.file_name().is_some_and(|p| p != "cargo-samply")
    }) {
        println!("{}", pth.file_name().unwrap().to_string_lossy());
        t = t.register_bin(
            pth.file_name().unwrap().to_string_lossy(),
            trycmd::schema::Bin::Path(pth.clone()),
        );
    }
    t.register_bin("cargo-samply", std::path::PathBuf::from(env!("CARGO_BIN_EXE_cargo-samply")));
}
