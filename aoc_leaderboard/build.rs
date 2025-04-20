fn main() {
    println!("cargo:rustc-check-cfg=cfg(use_doc_cfg)");
    if cfg!(docsrs) || rustversion::cfg!(nightly) {
        println!("cargo:rustc-cfg=use_doc_cfg");
    }
}
