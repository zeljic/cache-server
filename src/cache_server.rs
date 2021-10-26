#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CacheRequest {
	#[prost(string, tag = "1")]
	pub path: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CacheResponse {
	#[prost(bytes = "vec", tag = "1")]
	pub content: ::prost::alloc::vec::Vec<u8>,
}
#[doc = r" Generated server implementations."]
pub mod cache_service_server {
	#![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
	use tonic::codegen::*;
	#[doc = "Generated trait containing gRPC methods that should be implemented for use with CacheServiceServer."]
	#[async_trait]
	pub trait CacheService: Send + Sync + 'static {
		async fn get_content(
			&self,
			request: tonic::Request<super::CacheRequest>,
		) -> Result<tonic::Response<super::CacheResponse>, tonic::Status>;
		#[doc = "Server streaming response type for the GetContentStream method."]
		type GetContentStreamStream: futures_core::Stream<Item = Result<super::CacheResponse, tonic::Status>> + Send + 'static;
		async fn get_content_stream(
			&self,
			request: tonic::Request<super::CacheRequest>,
		) -> Result<tonic::Response<Self::GetContentStreamStream>, tonic::Status>;
	}
	#[derive(Debug)]
	pub struct CacheServiceServer<T: CacheService> {
		inner: _Inner<T>,
		accept_compression_encodings: (),
		send_compression_encodings: (),
	}
	struct _Inner<T>(Arc<T>);
	impl<T: CacheService> CacheServiceServer<T> {
		pub fn new(inner: T) -> Self {
			let inner = Arc::new(inner);
			let inner = _Inner(inner);
			Self {
				inner,
				accept_compression_encodings: Default::default(),
				send_compression_encodings: Default::default(),
			}
		}
		pub fn with_interceptor<F>(inner: T, interceptor: F) -> InterceptedService<Self, F>
		where
			F: tonic::service::Interceptor,
		{
			InterceptedService::new(Self::new(inner), interceptor)
		}
	}
	impl<T, B> tonic::codegen::Service<http::Request<B>> for CacheServiceServer<T>
	where
		T: CacheService,
		B: Body + Send + 'static,
		B::Error: Into<StdError> + Send + 'static,
	{
		type Response = http::Response<tonic::body::BoxBody>;
		type Error = Never;
		type Future = BoxFuture<Self::Response, Self::Error>;
		fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
			Poll::Ready(Ok(()))
		}
		fn call(&mut self, req: http::Request<B>) -> Self::Future {
			let inner = self.inner.clone();
			match req.uri().path() {
				"/cache_server.CacheService/GetContent" => {
					#[allow(non_camel_case_types)]
					struct GetContentSvc<T: CacheService>(pub Arc<T>);
					impl<T: CacheService> tonic::server::UnaryService<super::CacheRequest> for GetContentSvc<T> {
						type Response = super::CacheResponse;
						type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
						fn call(&mut self, request: tonic::Request<super::CacheRequest>) -> Self::Future {
							let inner = self.0.clone();
							let fut = async move { (*inner).get_content(request).await };
							Box::pin(fut)
						}
					}
					let accept_compression_encodings = self.accept_compression_encodings;
					let send_compression_encodings = self.send_compression_encodings;
					let inner = self.inner.clone();
					let fut = async move {
						let inner = inner.0;
						let method = GetContentSvc(inner);
						let codec = tonic::codec::ProstCodec::default();
						let mut grpc = tonic::server::Grpc::new(codec)
							.apply_compression_config(accept_compression_encodings, send_compression_encodings);
						let res = grpc.unary(method, req).await;
						Ok(res)
					};
					Box::pin(fut)
				}
				"/cache_server.CacheService/GetContentStream" => {
					#[allow(non_camel_case_types)]
					struct GetContentStreamSvc<T: CacheService>(pub Arc<T>);
					impl<T: CacheService> tonic::server::ServerStreamingService<super::CacheRequest> for GetContentStreamSvc<T> {
						type Response = super::CacheResponse;
						type ResponseStream = T::GetContentStreamStream;
						type Future = BoxFuture<tonic::Response<Self::ResponseStream>, tonic::Status>;
						fn call(&mut self, request: tonic::Request<super::CacheRequest>) -> Self::Future {
							let inner = self.0.clone();
							let fut = async move { (*inner).get_content_stream(request).await };
							Box::pin(fut)
						}
					}
					let accept_compression_encodings = self.accept_compression_encodings;
					let send_compression_encodings = self.send_compression_encodings;
					let inner = self.inner.clone();
					let fut = async move {
						let inner = inner.0;
						let method = GetContentStreamSvc(inner);
						let codec = tonic::codec::ProstCodec::default();
						let mut grpc = tonic::server::Grpc::new(codec)
							.apply_compression_config(accept_compression_encodings, send_compression_encodings);
						let res = grpc.server_streaming(method, req).await;
						Ok(res)
					};
					Box::pin(fut)
				}
				_ => Box::pin(async move {
					Ok(http::Response::builder()
						.status(200)
						.header("grpc-status", "12")
						.header("content-type", "application/grpc")
						.body(empty_body())
						.unwrap())
				}),
			}
		}
	}
	impl<T: CacheService> Clone for CacheServiceServer<T> {
		fn clone(&self) -> Self {
			let inner = self.inner.clone();
			Self {
				inner,
				accept_compression_encodings: self.accept_compression_encodings,
				send_compression_encodings: self.send_compression_encodings,
			}
		}
	}
	impl<T: CacheService> Clone for _Inner<T> {
		fn clone(&self) -> Self {
			Self(self.0.clone())
		}
	}
	impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
		fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
			write!(f, "{:?}", self.0)
		}
	}
	impl<T: CacheService> tonic::transport::NamedService for CacheServiceServer<T> {
		const NAME: &'static str = "cache_server.CacheService";
	}
}
