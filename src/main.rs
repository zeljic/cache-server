#[macro_use]
extern crate actix_web;

#[macro_use]
extern crate serde;

use actix_web::web::{Data, Query};
use actix_web::{App, HttpResponse, HttpServer};
use futures::lock::Mutex;
use futures::FutureExt;
use in_memory_cache::Cache;
use std::ops::DerefMut;

fn populate_cache<T>(_cache: &mut in_memory_cache::Cache, _key: T) -> Option<bytes::Bytes>
where
	T: Into<String>,
{
	todo!()
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

#[derive(Deserialize, Serialize, Debug)]
struct RequestQuery {
	path: String,
}

#[get("/cache/")]
async fn get_cache(
	query: Query<RequestQuery>,
	cache: Data<Mutex<in_memory_cache::Cache>>,
) -> actix_web::Result<actix_web::HttpResponse> {
	let mut value = None;

	cache
		.lock()
		.map(|mut cache| {
			value = get_cached_value(cache.deref_mut(), &query.path);
		})
		.await;

	match value {
		None => Ok(HttpResponse::NotFound().finish()),
		Some(content) => Ok(HttpResponse::Ok()
			.insert_header((actix_web::http::header::CONTENT_LENGTH, content.len()))
			.body(content)),
	}
}

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
	let cache = Data::new(Mutex::new(Cache::with_size_mb(32)));

	let http_init = move || {
		App::new()
			.app_data(Data::clone(&cache))
			.service(get_cache)
			.default_service(actix_web::web::route().to(|| HttpResponse::Ok().body("zdravo, svete")))
	};

	let http_server = HttpServer::new(http_init)
		.bind(("0.0.0.0", 1337))
		.map(|srv| {
			println!("http server is starting on {:?}", srv.addrs_with_scheme());

			srv
		})
		.map_err(|e| {
			eprintln!("{:?}", e);

			e
		})?
		.run();

	if let Err(e) = futures::try_join!(http_server) {
		println!("{:?}", e);
	}

	Ok(())
}
