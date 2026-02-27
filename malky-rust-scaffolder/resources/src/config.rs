#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub api_port: u16,
}

impl Config {
    pub fn from_env() -> Result<Self, std::env::VarError> {
        Ok(Self {
            database_url: std::env::var("DATABASE_URL")?,
            api_port: std::env::var("API_PORT")
                .unwrap_or_else(|_| "3001".to_string())
                .parse()
                .unwrap_or(3001),
        })
    }
}
