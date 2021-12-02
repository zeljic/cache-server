extern crate config as config_rs;

use self::config_rs::FileFormat;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Config {
	#[serde(rename = "http")]
	pub http: HttpServer,
	#[serde(rename = "grpc")]
	pub grpc: GRPCServer,
	#[serde(rename = "auth")]
	pub auth: AuthServer,
	pub cache: Cache,
}

impl Config {
	pub fn new() -> anyhow::Result<Self> {
		let mut config = config_rs::Config::default();

		if std::path::Path::new("Config.toml").exists() {
			if let Err(e) = config.merge(config_rs::File::new("Config.toml", FileFormat::Toml)) {
				error!("{:?}", anyhow::Error::new(e));
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
	pub token: String,
}

impl Default for AuthServer {
	fn default() -> Self {
		Self {
			url: String::from("http://auth"),
			token: String::new(),
		}
	}
}

/// Cache configuration
#[derive(Debug, Serialize, Deserialize)]
pub struct Cache {
	/// Cache size in megabytes
	pub size: usize,
}

/// Default cache configuration
impl Default for Cache {
	/// Default cache size is 128 MB
	fn default() -> Self {
		Self { size: 128 }
	}
}
