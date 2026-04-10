use std::{env, path::PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = PathBuf::from(env::var("OUT_DIR")?);
    let descriptor_path = out_dir.join("sword_descriptor_set.bin");

    tonic_prost_build::configure()
        .file_descriptor_set_path(&descriptor_path)
        .compile_protos(
            &["config/proto/users.proto", "config/proto/secure.proto"],
            &["config/proto"],
        )?;

    Ok(())
}
