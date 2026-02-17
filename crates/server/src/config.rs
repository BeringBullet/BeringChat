use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub server_name: String,
    pub base_url: String,
    pub database_path: String,
    pub admin_token: String,
    pub server_token: String,
    pub admin_username: String,
    pub admin_password: String,
    pub tenor_api_key: Option<String>,
}

impl Config {
    pub fn from_env() -> Self {
        let server_name = env::var("SERVER_NAME").unwrap_or_else(|_| "local".to_string());
        let base_url = env::var("BASE_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());
        let database_path = env::var("DATABASE_PATH").unwrap_or_else(|_| "./data.sqlite".to_string());
        let admin_token = env::var("ADMIN_TOKEN").unwrap_or_else(|_| "admin-token".to_string());
        let server_token = env::var("SERVER_TOKEN").unwrap_or_else(|_| "server-token".to_string());
        let admin_username = env::var("ADMIN_USERNAME").unwrap_or_else(|_| "admin".to_string());
        let admin_password = env::var("ADMIN_PASSWORD").unwrap_or_else(|_| "admin".to_string());
        let tenor_api_key = env::var("TENOR_API_KEY").ok().filter(|s| !s.is_empty());
        Self {
            server_name,
            base_url,
            database_path,
            admin_token,
            server_token,
            admin_username,
            admin_password,
            tenor_api_key,
        }
    }
}
