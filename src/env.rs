pub mod env {
    use std::env::{self, VarError};

    pub fn get_home_dir() -> Result<String, VarError> {
        let home_env_var_result = env::var("HOME");
        home_env_var_result
    }
}
