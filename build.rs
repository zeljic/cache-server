fn main() -> anyhow::Result<()> {
	tonic_build::configure()
		.out_dir("./src")
		.build_server(false)
		.build_client(true)
		.compile(&["./src/proto/auth.proto"], &["./src"])?;

	Ok(())
}
