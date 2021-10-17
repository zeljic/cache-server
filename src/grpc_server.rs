use actix_web::web::Data;
use futures::lock::Mutex;
use in_memory_cache::Cache;

pub async fn prepare_grpc_server(_cache: Data<Mutex<Cache>>) -> anyhow::Result<()> {
	Ok(())
}
