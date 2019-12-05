fn main() {
    tonic_build::configure()
        .build_client(true)
        .build_server(false)
        .format(true)
        .out_dir("src/pubsub/api")
        .compile(&["protos/google/pubsub/v1/pubsub.proto"], &["protos"])
        .unwrap();
    println!("cargo:rerun-if-changed=protos/google/pubsub/v1/pubsub.proto");

    tonic_build::configure()
        .build_client(true)
        .build_server(false)
        .format(true)
        .out_dir("src/datastore/api")
        .compile(&["protos/google/datastore/v1/datastore.proto"], &["protos"])
        .unwrap();
    println!("cargo:rerun-if-changed=protos/google/datastore/v1/datastore.proto");

    //? Disabled due to bug in `prost` that generates multiple client modules for a single package.
    // tonic_build::configure()
    //     .build_client(true)
    //     .build_server(false)
    //     .format(true)
    //     .out_dir("src/vision/api")
    //     .compile(&["protos/google/cloud/vision/v1/image_annotator.proto"], &["protos"])
    //     .unwrap();
    // println!("cargo:rerun-if-changed=protos/google/cloud/vision/v1/image_annotator.proto");
}
