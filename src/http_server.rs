use crate::get_cached_value;

use actix_web::web::{Data, Query};
use actix_web::{HttpResponse, HttpServer};
use futures::lock::Mutex;
use in_memory_cache::Cache;
use std::ops::DerefMut;
use std::str::FromStr;

use crate::auth::authentication_service_client::AuthenticationServiceClient;
use crate::auth::SessionRequest;
use actix_web::dev::Service;

use actix_web::App;
use futures::FutureExt;

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

async fn check_session(token: String, config: Data<crate::config::Config>) -> anyhow::Result<bool> {
	let url: &str = config.auth_server.url.as_str();

	let endpoint = tonic::transport::channel::Endpoint::from_str(url).map_err(|e| {
		eprintln!("{:?}", e);
		anyhow::Error::new(e).context("Unable to create endpoint from url")
	})?;

	let mut client = AuthenticationServiceClient::connect(endpoint)
		.await
		.map_err(|e| anyhow::Error::new(e).context("Unable to connect to provided gRPC host:port"))?;

	let response = client
		.is_session_valid(tonic::Request::new(SessionRequest { token }))
		.await
		.map_err(|e| anyhow::Error::new(e).context("gRPC service is_session_valid returns error"))?
		.into_inner();

	Ok(response.valid)
}

pub async fn prepare_http_server(cache: Data<Mutex<Cache>>, config: Data<crate::config::Config>) -> anyhow::Result<()> {
	let local_config = Data::clone(&config);

	let http_init = move || {
		let config = Data::clone(&config);

		App::new()
			.app_data(Data::clone(&cache))
			.wrap_fn(move |req, srv| {
				let async_config = Data::clone(&config);

				let token: Option<String> = req
					.cookie(&async_config.auth_server.token_key)
					.map(|cookie| cookie.value().to_owned());

				let fut = srv.call(req);

				async {
					if let Some(token) = token {
						if let Ok(valid) = check_session(token, async_config).await {
							if valid {
								if let Ok(res) = fut.await {
									return Ok(res);
								}
							}
						}
					}

					Err(actix_web::error::ErrorUnauthorized("Unauthorized"))
				}
			})
			.service(get_cache)
			.default_service(actix_web::web::route().to(|| HttpResponse::Ok().body("zdravo, svete")))
	};

	HttpServer::new(http_init)
		.bind((local_config.http_server.host.as_str(), local_config.http_server.port))
		.map(|srv| {
			println!("http server is starting on {:?}", srv.addrs_with_scheme());

			srv
		})
		.map_err(|e| {
			eprintln!("{:?}", e);

			e
		})?
		.run()
		.await?;

	Ok(())
}
