extern crate config as config_rs;

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
		let mut builder = config_rs::Config::builder();

		let file = std::path::Path::new("Config.toml");

		if file.exists() {
			builder = builder.add_source(config_rs::File::from(file));
		}

		builder = builder.add_source(
			config_rs::Environment::with_prefix("CACHE_SERVER")
				.separator("_")
				.try_parsing(true),
		);

		match builder.build() {
			Ok(built) => match built.try_deserialize::<Config>() {
				Ok(config) => Ok(config),
				Err(e) => Err(anyhow::Error::new(e).context("Unable to deserialize configuration")),
			},
			Err(e) => Err(anyhow::Error::new(e).context("Unable to build configuration")),
		}
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
	pub enable: bool,
	pub url: String,
	pub token: String,
}

impl Default for AuthServer {
	fn default() -> Self {
		Self {
			enable: false,
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
