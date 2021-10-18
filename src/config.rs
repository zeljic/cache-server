extern crate config as config_rs;

use self::config_rs::FileFormat;

#[derive(Debug, Serialize, Deserialize, Default)]
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

		if std::path::Path::new("Config.yaml").exists() {
			if let Err(e) = config.merge(config_rs::File::new("Config.yaml", FileFormat::Yaml)) {
				eprintln!("{:?}", anyhow::Error::new(e));
			}
		}

		if std::path::Path::new("Config.toml").exists() {
			if let Err(e) = config.merge(config_rs::File::new("Config.toml", FileFormat::Toml)) {
				eprintln!("{:?}", anyhow::Error::new(e));
			}
		}

		config.merge(config_rs::Environment::new().prefix("CACHE_SERVER").separator("_"))?;

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

impl Default for HttpServer {
	fn default() -> Self {
		Self {
			host: String::from("0.0.0.0"),
			port: 1337,
		}
	}
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GRPCServer {
	pub host: String,
	pub port: u16,
}

impl Default for GRPCServer {
	fn default() -> Self {
		Self {
			host: String::from("[::1]"),
			port: 1338,
		}
	}
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthServer {
	pub url: String,
	#[serde(rename = "token-key")]
	pub token_key: String,
}

impl Default for AuthServer {
	fn default() -> Self {
		Self {
			url: String::from("http://auth"),
			token_key: String::new(),
		}
	}
}
