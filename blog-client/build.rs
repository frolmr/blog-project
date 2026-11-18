use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../blog-proto/proto");
    let proto_file = proto_dir.join("blog.proto");

    println!("cargo:rerun-if-changed={}", proto_file.display());

    tonic_prost_build::configure()
        .build_client(true)
        .build_server(false)
        .compile_protos(&[proto_file], &[proto_dir])?;

    Ok(())
}
