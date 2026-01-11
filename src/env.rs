use std::env::{self, VarError};

pub fn get_home_dir() -> Result<String, VarError> {
    env::var("HOME")
}
