#[macro_use]
extern crate actix_web;

#[macro_use]
extern crate serde;

#[macro_use]
extern crate log;

extern crate tonic;

use actix_web::web::Data;
use env_logger::WriteStyle;
use std::str::FromStr;

use crate::auth::SessionRequest;
use futures::{lock::Mutex, try_join};
use log::LevelFilter;

use crate::auth::authentication_service_client::AuthenticationServiceClient;

mod auth;
mod cache_server;
mod config;
mod grpc_server;
mod http_server;

fn populate_cache<T>(_cache: &mut in_memory_cache::Cache, _key: T) -> Option<bytes::Bytes>
where
	T: Into<String>,
{
	None
}

fn get_cached_value<T>(cache: &mut in_memory_cache::Cache, key: T) -> Option<bytes::Bytes>
where
	T: Into<String>,
{
	let key: String = key.into();
	let key: &str = &key;

	let mut value: Option<bytes::Bytes> = cache.get_bytes(key);

	if value.is_none() {
		value = populate_cache(cache, key);
	}

	value
}

pub async fn check_session(token: &str, config: Data<crate::config::Config>) -> anyhow::Result<bool> {
	let url: &str = config.auth.url.as_str();

	let endpoint = tonic::transport::channel::Endpoint::from_str(url).map_err(|e| {
		error!("{:?}", e);
		anyhow::Error::new(e).context("Unable to create endpoint from url")
	})?;

	let mut client = AuthenticationServiceClient::connect(endpoint)
		.await
		.map_err(|e| anyhow::Error::new(e).context("Unable to connect to provided gRPC host:port"))?;

	let response = client
		.is_session_valid(tonic::Request::new(SessionRequest { token: token.to_owned() }))
		.await
		.map_err(|e| anyhow::Error::new(e).context("gRPC service is_session_valid returns error"))?
		.into_inner();

	Ok(response.valid)
}

pub fn check_session_blocking(token: &str, config: Data<crate::config::Config>) -> anyhow::Result<bool> {
	futures::executor::block_on(check_session(token, config))
}

fn init_logging() {
	let log_level = match std::env::var("CACHE_SERVER_LOG_LEVEL") {
		Ok(level) => level,
		Err(_) => String::from("ERROR"),
	};

	if let Ok(level) = LevelFilter::from_str(&log_level) {
		env_logger::Builder::new()
			.filter_level(level)
			.write_style(WriteStyle::Auto)
			.init();
	}
}

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
	init_logging();

	let config = Data::new(crate::config::Config::new()?);
	let cache = Data::new(Mutex::new(in_memory_cache::Cache::with_size_mb(1)));

	{
		let mut cache = cache.lock().await;

		cache.add("/dog.jpg", std::fs::read("/home/zeljic/Desktop/dog.jpg")?)?;
	}

	let http_server = crate::http_server::prepare_http_server(Data::clone(&cache), Data::clone(&config));
	let grpc_server = crate::grpc_server::prepare_grpc_server(Data::clone(&cache), Data::clone(&config));

	if let Err(e) = try_join!(http_server, grpc_server) {
		error!("{:?}", e);
	}

	Ok(())
}
