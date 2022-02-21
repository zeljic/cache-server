use actix_web::{
	web::{Data, Query},
	HttpResponse, HttpServer, Responder,
};
use futures::lock::Mutex;
use in_memory_cache::Cache;
use std::ops::DerefMut;

use actix_web::dev::Service;

use actix_web::App;

#[derive(Deserialize, Serialize, Debug)]
struct RequestQuery {
	path: String,
}

#[get("/cache/")]
async fn get_cache(
	query: Query<RequestQuery>,
	cache: Data<Mutex<in_memory_cache::Cache>>,
) -> actix_web::Result<actix_web::HttpResponse> {
	match async {
		let mut cache = cache.lock().await;

		crate::get_cached_value(cache.deref_mut(), &query.path).await
	}
	.await
	{
		None => Ok(HttpResponse::NotFound().finish()),
		Some(content) => Ok(HttpResponse::Ok()
			.insert_header((actix_web::http::header::CONTENT_LENGTH, content.len()))
			.body(content)),
	}
}

async fn default_handler() -> actix_web::Result<impl Responder> {
	Ok(HttpResponse::Ok().body("zdravo, svete!"))
}

pub async fn prepare_http_server(cache: Data<Mutex<Cache>>, config: Data<crate::config::Config>) -> anyhow::Result<()> {
	let local_config = Data::clone(&config);

	let http_init = move || {
		let config = Data::clone(&config);

		App::new()
			.app_data(Data::clone(&cache))
			.wrap_fn(move |req, srv| {
				let conf = Data::clone(&config);

				let token: Option<String> = req.cookie(&conf.auth.token).map(|cookie| cookie.value().to_owned());

				let fut = srv.call(req);

				async {
					if let Some(token) = token {
						if let Ok(valid) = crate::check_session(&token, conf).await {
							if valid {
								if let Ok(res) = fut.await {
									return Ok(res);
								}
							}
						}
					}

					warn!("Unauthorized access!");

					Err(actix_web::error::ErrorUnauthorized("Unauthorized"))
				}
			})
			.service(get_cache)
			.default_service(actix_web::web::to(default_handler))
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
