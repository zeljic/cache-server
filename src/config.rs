extern crate config as config_rs;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
	#[serde(rename = "http-server")]
	pub http_server: HttpServer,
	#[serde(rename = "grpc-server")]
	pub grpc_server: GRPCServer,
	#[serde(rename = "auth-server")]
	pub auth_server: AuthServer,
}

impl Config {
	pub fn new() -> anyhow::Result<Self> {
		let mut config = config_rs::Config::default();

		config
			.merge(config_rs::File::with_name("Config.toml"))?
			.merge(config_rs::Environment::new().separator("_").prefix("CACHE_SERVER"))?;

		config
			.try_into()
			.map_err(|e| anyhow::Error::new(e).context("Unable to configure"))
	}
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HttpServer {
	pub host: String,
	pub port: u16,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GRPCServer {
	pub host: String,
	pub port: u16,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthServer {
	pub url: String,
	#[serde(rename = "token-key")]
	pub token_key: String,
}
