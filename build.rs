use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let proto_root = "protos";
    let proto_out = Path::new(&out_dir).join("proto");
    fs::create_dir_all(&proto_out).unwrap();
    protobuf_build::Builder::new()
        .includes(&["protos".to_owned()])
        .files(&["protos/geoip2.proto"])
        .out_dir(proto_out.as_path().display().to_string())
        .generate();
    println!("cargo:rerun-if-changed={}", proto_root);
}
