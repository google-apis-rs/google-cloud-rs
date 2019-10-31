fn main() {
    tonic_build::configure()
        .build_client(true)
        .build_server(false)
        .format(true)
        .out_dir("src/pubsub/api")
        .compile(&["protos/google/pubsub/v1/pubsub.proto"], &["protos"])
        .unwrap();
    println!("cargo:rerun-if-changed=protos/google/pubsub/v1/pubsub.proto");
}
