fn main() {
    println!("cargo:rerun-if-changed=assets/wsmf.rc");
    println!("cargo:rerun-if-changed=assets/wsmf.ico");
    println!("cargo:rerun-if-changed=assets/wsmf.manifest");
    embed_resource::compile("assets/wsmf.rc", embed_resource::NONE)
        .manifest_required()
        .unwrap();
}
