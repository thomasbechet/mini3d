use cbindgen::Config;

fn main() {
    cbindgen::Builder::new()
		.with_crate(".")
		.with_config(Config::from_file("cbindgen.toml").unwrap())
		.generate()
		.expect("Unable to generate bindings")
		.write_to_file("mini3d.h");
}
