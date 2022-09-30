use crate::cache_server::{
	cache_service_server::{CacheService, CacheServiceServer},
	CacheRequest, CacheResponse,
};
use actix_web::web::Data;
use futures::{lock::Mutex, Stream, StreamExt};
use in_memory_cache::Cache;

use std::{ops::DerefMut, pin::Pin};

#[cfg(unix)]
use tokio::signal::unix::{signal, SignalKind};

use tonic::{service::Interceptor, Request, Response, Status};

type PinBoxResponse<T> = Pin<Box<dyn Stream<Item = Result<T, Status>> + Send + Sync>>;

struct CacheServerService {
	cache: Data<Mutex<Cache>>,
}

#[tonic::async_trait]
impl CacheService for CacheServerService {
	async fn get_content(&self, request: Request<CacheRequest>) -> Result<Response<CacheResponse>, Status> {
		let path: &str = request.get_ref().path.as_str();

		return match async {
			let mut cache = self.cache.lock().await;

			crate::get_cached_value(cache.deref_mut(), path).await
		}
		.await
		{
			Some(content) => {
				let response = CacheResponse {
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
		request: Request<CacheRequest>,
	) -> Result<Response<Self::GetContentStreamStream>, Status> {
		let path = request.get_ref().path.to_owned();

		let cache = Data::clone(&self.cache);
		let (tx, rx) = tokio::sync::mpsc::channel(4);

		tokio::spawn(async move {
			if let Some(content) = async {
				let mut cache = cache.lock().await;

				crate::get_cached_value(cache.deref_mut(), &path).await
			}
			.await
			{
				let mut stream = tokio_stream::iter(content.chunks(4 * 1024));

				while let Some(chunk) = stream.next().await {
					if tx.send(Ok(CacheResponse { content: chunk.to_vec() })).await.is_ok() {}
				}
			}
		});

		Ok(Response::new(Box::pin(tokio_stream::wrappers::ReceiverStream::new(rx))))
	}
}

#[derive(Clone)]
struct AuthenticationInterceptor {
	config: Data<crate::config::Config>,
}

impl AuthenticationInterceptor {
	fn new(config: Data<crate::config::Config>) -> Self {
		Self { config }
	}
}

impl Interceptor for AuthenticationInterceptor {
	fn call(&mut self, req: Request<()>) -> Result<Request<()>, Status> {
		let config = Data::clone(&self.config);

		if config.auth.enable {
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
		} else {
			Ok(req)
		}
	}
}

pub async fn prepare_grpc_server(cache: Data<Mutex<Cache>>, config: Data<crate::config::Config>) -> anyhow::Result<()> {
	let addr = format!("{}:{}", config.grpc.host, config.grpc.port).parse()?;

	let cache_server_service = CacheServerService {
		cache: Data::clone(&cache),
	};

	let auth_intercept = AuthenticationInterceptor::new(config);

	tonic::transport::Server::builder()
		.add_service(CacheServiceServer::with_interceptor(cache_server_service, auth_intercept))
		.serve_with_shutdown(addr, async {
			#[cfg(unix)]
			{
				if let (Ok(mut sigint), Ok(mut sigterm)) = (signal(SignalKind::interrupt()), signal(SignalKind::terminate())) {
					tokio::select! {
						_ = sigint.recv() => {
							info!("Signal interrupt (SIGINT) is received");
						},
						_ = sigterm.recv() => {
							info!("Signal terminate (SIGTERM) is received");
						}
					}
				}
			}

			#[cfg(windows)]
			{
				if let Err(e) = tokio::spawn(tokio::signal::ctrl_c()).await {
					error!("{:?}", e);
				}

				info!("Wait for SIGINT (CTRL+C) to close grpc Server");
			}

			info!("gRPC server is shutting down");
		})
		.await?;

	Ok(())
}
