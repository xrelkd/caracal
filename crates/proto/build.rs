use prost_build::Config;

fn prost_config() -> Config {
    let mut config = Config::new();
    let _ = config.protoc_arg("--experimental_allow_proto3_optional");
    config
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure().compile_protos_with_config(
        prost_config(),
        &["proto/System.proto", "proto/Task.proto"],
        &["proto/"],
    )?;
    Ok(())
}
