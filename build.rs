fn main() -> anyhow::Result<()> {
	tonic_build::configure()
		.out_dir("./src")
		.build_server(false)
		.build_client(true)
		.compile(&["./src/proto/auth.proto"], &["./src"])?;

	tonic_build::configure()
		.out_dir("./src")
		.build_server(true)
		.build_client(false)
		.compile(&["./src/proto/cache-server.proto"], &["./src"])?;

	Ok(())
}
