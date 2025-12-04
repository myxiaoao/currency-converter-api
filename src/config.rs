use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub server_host: String,
    pub server_port: u16,
    pub redis_url: String,
    pub ecb_url: String,
    pub update_cron: String,
}

impl Config {
    pub fn from_env() -> Result<Self, env::VarError> {
        Ok(Config {
            server_host: env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            server_port: env::var("SERVER_PORT")
                .unwrap_or_else(|_| "3000".to_string())
                .parse()
                .expect("SERVER_PORT must be a valid u16"),
            redis_url: env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
            ecb_url: env::var("ECB_URL").unwrap_or_else(|_| {
                "https://www.ecb.europa.eu/stats/eurofxref/eurofxref-daily.xml".to_string()
            }),
            update_cron: env::var("UPDATE_CRON").unwrap_or_else(|_| "0 0 15 * * *".to_string()),
        })
    }

    pub fn server_address(&self) -> String {
        format!("{}:{}", self.server_host, self.server_port)
    }
}
