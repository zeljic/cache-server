use crate::get_cached_value;

use actix_web::web::{Data, Query};
use actix_web::{HttpResponse, HttpServer};
use futures::lock::Mutex;
use in_memory_cache::Cache;
use std::ops::DerefMut;

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

async fn check_session(token: String) -> anyhow::Result<bool> {
	let mut client = AuthenticationServiceClient::connect("http://[::1]:5055")
		.await
		.map_err(|e| anyhow::Error::new(e).context("Unable to connect to provided gRPC host:port"))?;

	let response = client
		.is_session_valid(tonic::Request::new(SessionRequest { token }))
		.await
		.map_err(|e| anyhow::Error::new(e).context("gRPC service is_session_valid returns error"))?
		.into_inner();

	Ok(response.valid)
}

pub async fn prepare_http_server(cache: Data<Mutex<Cache>>) -> anyhow::Result<()> {
	let http_init = move || {
		App::new()
			.app_data(Data::clone(&cache))
			.wrap_fn(|req, srv| {
				let token = req.cookie("x-token").map(|cookie| cookie.value().to_owned());

				let fut = srv.call(req);

				async {
					if let Some(token) = token {
						if let Ok(valid) = check_session(token).await {
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
		.bind(("0.0.0.0", 1337))
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
