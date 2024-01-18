use std::io;
use std::path::PathBuf;
use std::result;
use std::str::Utf8Error;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("{path}: {source}")]
    PathIo { source: io::Error, path: PathBuf },
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    Logger(#[from] log::SetLoggerError),
    #[error(transparent)]
    Utf8(#[from] Utf8Error),
    #[error(transparent)]
    TomlDeserialization(#[from] toml::de::Error),
    #[error(transparent)]
    TomlManifest(#[from] cargo_toml::Error),
    #[error("--bin and --example are mutually exclusive")]
    BinAndExampleMutuallyExclusive,
    #[error("Build failed")]
    CargoBuildFailed,
    #[error("No binary found in 'Cargo.toml'")]
    NoBinaryFound,
    #[error("The binary to run can't be determined. Use the `--bin` option to specify a binary, or the `default-run` manifest key.")]
    BinaryToRunNotDetermined,
    #[error("Failed to locate project")]
    CargoLocateProjectFailed,
}

/// Alias for a `Result` with the error type `hld::Error`.
pub type Result<T> = result::Result<T, Error>;

/// Extension trait for `io::Result`.
pub trait IOResultExt<T> {
    fn path_ctx<P: Into<PathBuf>>(self, path: P) -> Result<T>;
}

impl<T> IOResultExt<T> for io::Result<T> {
    fn path_ctx<P: Into<PathBuf>>(self, path: P) -> Result<T> {
        self.map_err(|source| Error::PathIo {
            source,
            path: path.into(),
        })
    }
}
