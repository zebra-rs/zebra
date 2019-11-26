fn main() {
    extern crate pnet_macros;
    extern crate syntex;
    extern crate glob;

    use std::env;
    use std::path::Path;

    // globbing for files to pre-process:
    let pattern = "./src/bgp/packet/**/*.rs.in";
    for entry in glob::glob(pattern).expect("Failed to read glob pattern") {
        if let Ok(path) = entry {
            let src = Path::new(path.to_str().expect("Invalid src Specified."));
            let out_dir = env::var_os("OUT_DIR").expect("Invalid OUT_DIR.");
            let file = Path::new(path.file_stem().expect("Invalid file_stem."));
            let dst = Path::new(&out_dir).join(file);
            let mut registry = syntex::Registry::new();
            pnet_macros::register(&mut registry);
            registry.expand("", &src, &dst).unwrap();
        }
    }
}
