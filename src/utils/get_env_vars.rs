use std::env;

pub fn get_env_var(key: &str) -> String {
    env::var(key).expect("Key not found in .env file")
}
