use crate::cache_server::{
	cache_service_server::{CacheService, CacheServiceServer},
	CacheRequest, CacheResponse,
};
use actix_web::web::Data;
use futures::{lock::Mutex, Stream};
use in_memory_cache::Cache;

use std::{ops::DerefMut, pin::Pin};

use tonic::{Request, Response, Status};

type PinBoxResponse<T> = Pin<Box<dyn Stream<Item = Result<T, Status>> + Send + Sync>>;

struct CacheServerService {
	cache: Data<futures::lock::Mutex<in_memory_cache::Cache>>,
}

#[tonic::async_trait]
impl CacheService for CacheServerService {
	async fn get_content(&self, request: Request<CacheRequest>) -> Result<Response<CacheResponse>, Status> {
		let path: &str = request.get_ref().path.as_str();

		return match async {
			let mut cache = self.cache.lock().await;

			crate::get_cached_value(cache.deref_mut(), path)
		}
		.await
		{
			Some(content) => {
				let response = crate::cache_server::CacheResponse {
					content: content.to_vec(),
				};

				Ok(Response::new(response))
			}
			None => Err(Status::not_found("Cache item is not found")),
		};
	}

	type GetContentStreamStream = PinBoxResponse<CacheResponse>;

	async fn get_content_stream(
		&self,
		_request: Request<CacheRequest>,
	) -> Result<Response<Self::GetContentStreamStream>, Status> {
		todo!()
	}
}

fn auth_intercept(req: Request<()>, config: Data<crate::config::Config>) -> anyhow::Result<Request<()>, Status> {
	if let Some(token) = req.metadata().get("token") {
		if let Ok(token) = String::from_utf8(token.as_bytes().to_vec()) {
			if let Ok(valid) = crate::check_session_blocking(&token, config) {
				return if valid {
					Ok(req)
				} else {
					Err(Status::unauthenticated("Invalid token"))
				};
			}
		}
	}

	Err(Status::unauthenticated("Invalid session"))
}

pub async fn prepare_grpc_server(cache: Data<Mutex<Cache>>, config: Data<crate::config::Config>) -> anyhow::Result<()> {
	let addr = format!("{}:{}", config.grpc.host, config.grpc.port).parse()?;

	let cache_server_service = CacheServerService {
		cache: Data::clone(&cache),
	};

	tonic::transport::Server::builder()
		.add_service(CacheServiceServer::with_interceptor(cache_server_service, move |req| {
			auth_intercept(req, Data::clone(&config))
		}))
		.serve_with_shutdown(addr, async {
			info!("Wait for CTRL+C to close grpc Server");

			if let Err(e) = tokio::spawn(tokio::signal::ctrl_c()).await {
				error!("{:?}", e);
			}

			info!("gRPC server is shutting down");
		})
		.await?;

	Ok(())
}
