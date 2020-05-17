use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let protos = [
        (["protos/google/pubsub/v1/pubsub.proto"], "src/pubsub/api"),
        (
            ["protos/google/datastore/v1/datastore.proto"],
            "src/datastore/api",
        ),
        (
            ["protos/google/cloud/vision/v1/image_annotator.proto"],
            "src/vision/api",
        ),
    ];

    for (proto_files, out_dir) in protos.iter() {
        fs::create_dir_all(&out_dir)?;

        tonic_build::configure()
            .build_client(true)
            .build_server(false)
            .format(true)
            .out_dir(&out_dir)
            .compile(proto_files, &["protos"])?;

        for file in proto_files {
            println!("cargo:rerun-if-changed={}", &file);
        }
    }

    Ok(())
}
