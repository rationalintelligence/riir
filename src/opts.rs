use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
pub struct Opts {
    /// A path to a file to refactor.
    pub file: PathBuf,
}
