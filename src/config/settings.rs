use std::env;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub default_provider: String,
    pub phimapi_base_url: String,
    pub nguonc_base_url: String,
    pub vsmov_base_url: String,
    pub supabase_url: String,
    pub supabase_anon_key: String,
    pub request_timeout_secs: u64,
    pub enable_image_cache: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            default_provider: "phimapi".to_string(),
            phimapi_base_url: "https://phimapi.com".to_string(),
            nguonc_base_url: "https://phim.nguonc.com/api".to_string(),
            vsmov_base_url: "https://vsmov.com/api".to_string(),
            supabase_url: "https://uvzdtpkzixllhzinhtgx.supabase.co".to_string(),
            supabase_anon_key: "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZSIsInJlZiI6InV2emR0cGt6aXhsbGh6aW5odGd4Iiwicm9sZSI6ImFub24iLCJpYXQiOjE3OTIwNTk2MTUsImV4cCI6MjEwNzYzNTYxNX0.iNSmUNSX2OW-YvcQm5XDVl5LRcQUMyu9iSdYTVhcJRI".to_string(),
            request_timeout_secs: 6,
            enable_image_cache: true,
        }
    }
}

impl AppConfig {
    pub fn from_env() -> Self {
        // Attempt to load .env file if it exists
        let _ = dotenvy::dotenv();

        let default_provider = env::var("DEFAULT_API_PROVIDER").unwrap_or_else(|_| "phimapi".to_string());
        let phimapi_base_url = env::var("PHIMAPI_BASE_URL").unwrap_or_else(|_| "https://phimapi.com".to_string());
        let nguonc_base_url = env::var("NGUONC_BASE_URL").unwrap_or_else(|_| "https://phim.nguonc.com/api".to_string());
        let vsmov_base_url = env::var("VSMOV_BASE_URL").unwrap_or_else(|_| "https://vsmov.com/api".to_string());
        let supabase_url = env::var("SUPABASE_URL")
            .or_else(|_| env::var("PUBLIC_SUPABASE_URL"))
            .unwrap_or_else(|_| "https://uvzdtpkzixllhzinhtgx.supabase.co".to_string());
        let supabase_anon_key = env::var("SUPABASE_ANON_KEY")
            .or_else(|_| env::var("PUBLIC_SUPABASE_ANON_KEY"))
            .unwrap_or_else(|_| "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZSIsInJlZiI6InV2emR0cGt6aXhsbGh6aW5odGd4Iiwicm9sZSI6ImFub24iLCJpYXQiOjE3OTIwNTk2MTUsImV4cCI6MjEwNzYzNTYxNX0.iNSmUNSX2OW-YvcQm5XDVl5LRcQUMyu9iSdYTVhcJRI".to_string());
        let request_timeout_secs = env::var("REQUEST_TIMEOUT")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(6);
        let enable_image_cache = env::var("ENABLE_IMAGE_CACHE")
            .map(|v| v.to_lowercase() == "true" || v == "1")
            .unwrap_or(true);

        Self {
            default_provider,
            phimapi_base_url,
            nguonc_base_url,
            vsmov_base_url,
            supabase_url,
            supabase_anon_key,
            request_timeout_secs,
            enable_image_cache,
        }
    }
}
