use actix_web::{web, web::Data, HttpRequest, HttpResponse, HttpServer, Responder};
use futures::lock::Mutex;
use in_memory_cache::Cache;

use actix_web::dev::Service;

use actix_web::{http::header::TryIntoHeaderPair, App};

use futures::StreamExt;

#[inline]
fn header_key<T: Into<String>>(key: T) -> impl TryIntoHeaderPair {
	("x-cache-server-key", key.into())
}

#[get("/get/{key}")]
async fn get_cache(path: web::Path<String>, cache: Data<Mutex<Cache>>) -> actix_web::Result<HttpResponse> {
	let key = path.into_inner();

	match async { cache.lock().await.get_bytes(key.to_owned()) }.await {
		None => Ok(HttpResponse::NotFound().insert_header(header_key(key)).finish()),
		Some(content) => Ok(HttpResponse::Ok()
			.insert_header((actix_web::http::header::CONTENT_LENGTH, content.len()))
			.body(content)),
	}
}

#[get("/get")]
async fn get_cache_key(request: HttpRequest, cache: Data<Mutex<Cache>>) -> actix_web::Result<HttpResponse> {
	match get_header(&request, "x-cache-server-key") {
		Some(key) => match async { cache.lock().await.get_bytes(&key) }.await {
			None => Ok(HttpResponse::NotFound().insert_header(header_key(&key)).finish()),
			Some(content) => Ok(HttpResponse::Ok()
				.insert_header(header_key(&key))
				.insert_header((actix_web::http::header::CONTENT_LENGTH, content.len()))
				.body(content)),
		},
		None => Err(actix_web::error::ErrorBadRequest("No key")),
	}
}

async fn set(key: &str, mut body: web::Payload, cache: Data<Mutex<Cache>>) -> anyhow::Result<()> {
	let mut bytes = bytes::BytesMut::new();

	while let Some(item) = body.next().await {
		bytes.extend_from_slice(&item?);
	}

	cache
		.lock()
		.await
		.add(key, bytes)
		.map_err(|e| anyhow::Error::msg(e.to_string()))
}

#[post("/set/{key}")]
async fn set_cache_key(
	path: web::Path<String>,
	body: web::Payload,
	cache: Data<Mutex<Cache>>,
) -> actix_web::Result<HttpResponse> {
	let key: String = path.into_inner();

	match set(&key, body, cache).await {
		Ok(_) => Ok(HttpResponse::Ok().insert_header(header_key(key)).finish()),
		Err(e) => {
			error!("{:?}", e);
			Err(actix_web::error::ErrorBadRequest(e))
		}
	}
}

#[post("/set")]
async fn set_cache(request: HttpRequest, body: web::Payload, cache: Data<Mutex<Cache>>) -> actix_web::Result<HttpResponse> {
	match get_header(&request, "x-cache-server-key") {
		Some(key) => match set(&key, body, cache).await {
			Ok(_) => Ok(HttpResponse::Ok().insert_header(header_key(key)).finish()),
			Err(e) => {
				error!("{:?}", e);

				Err(actix_web::error::ErrorInternalServerError("Unable to set key"))
			}
		},
		None => Err(actix_web::error::ErrorBadRequest("No key")),
	}
}

async fn default_handler() -> actix_web::Result<impl Responder> {
	Ok(HttpResponse::Ok().body("zdravo, svete!"))
}

fn get_header(request: &HttpRequest, key: &str) -> Option<String> {
	let mut token = None;

	if let Some(t) = request.headers().get(key) {
		if let Ok(t) = t.to_str() {
			token = Some(t.to_owned());
		}
	}

	token
}

pub async fn prepare_http_server(cache: Data<Mutex<Cache>>, config: Data<crate::config::Config>) -> anyhow::Result<()> {
	let local_config = Data::clone(&config);

	let http_init = move || {
		let config = Data::clone(&config);

		App::new()
			.app_data(Data::clone(&cache))
			.wrap_fn(move |service_request, app_routing| {
				let conf = Data::clone(&config);

				let token = get_header(service_request.request(), &conf.auth.token);

				let fut = app_routing.call(service_request);

				async {
					if conf.auth.enable {
						if let Some(token) = token {
							if let Ok(valid) = crate::check_session(&token, conf).await {
								if valid {
									return fut.await;
								}
							}
						}

						warn!("Unauthorized access!");

						Err(actix_web::error::ErrorUnauthorized("Unauthorized"))
					} else {
						fut.await
					}
				}
			})
			.service(get_cache)
			.service(get_cache_key)
			.service(set_cache_key)
			.service(set_cache)
			.default_service(web::to(default_handler))
	};

	HttpServer::new(http_init)
		.bind((local_config.http.host.as_str(), local_config.http.port))
		.map(|srv| {
			info!("http server is starting on {:?}", srv.addrs_with_scheme());

			srv
		})
		.map_err(|e| {
			error!("{:?}", e);

			e
		})?
		.run()
		.await?;

	Ok(())
}
