use std::env;
// use std::fs;
use std::path::PathBuf;
// use std::process::Command;
// use pkg_config;
use metadeps;
fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let libs = metadeps::probe().unwrap();
    // eprintln!("{:?}", &libs);

    let sofia_sip_ua_include_paths = &libs["sofia-sip-ua"].include_paths;
    // eprintln!("{:?}", sofia_sip_ua_include_paths);

    // std::process::exit(1);

    // pkg_config::Config::new()
    //     .atleast_version("1.12")
    //     .probe("sofia-sip-ua")
    //     .unwrap();
    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header("wrapper.h")
        /* su init / deinit */
        .allowlist_function("su_init")
        .allowlist_function("su_deinit")
        /* su home init / deinit */
        .allowlist_function("su_home_init")
        .allowlist_function("su_home_deinit")
        /* su root class */
        .allowlist_function("su_root_create")
        .allowlist_function("su_root_step")
        .allowlist_function("su_root_threading")
        .allowlist_function("su_root_destroy")
        .allowlist_function("nua_create")
        .allowlist_function("nua_handle")
        .allowlist_function("nua_shutdown")
        .allowlist_function("nua_destroy")
        /* nua class */
        .allowlist_function("nua_set_params")
        .opaque_type("su_home_t")
        .opaque_type("su_root_t")
        .opaque_type("sip_t")
        /* tags */
        .allowlist_type("tagi_t")
        .allowlist_type("tag_type_t")
        .allowlist_type("tag_value_t")
        .allowlist_var("tag_null")
        .allowlist_var("tag_skip")
        .allowlist_var("tag_next")
        .allowlist_var("tag_any")
        .allowlist_var("tag_filter")
        /* nua tags */
        .allowlist_var("nutag_url")
        /* libc */
        .allowlist_function("atexit")
        // .opaque_type()
        // .allowlist_function("nua_set_params")
        // .allowlist_function("su_root_run")
        // .allowlist_function("nua_destroy")
        // .allowlist_function("su_root_destroy")
        // .allowlist_function("su_home_deinit")
        // .allowlist_function("su_deinit")
        // .allowlist_type("su_root_t")
        // .allowlist_type("msg_common_t")
        // .allowlist_type("nua_t")
        .clang_args(
            sofia_sip_ua_include_paths
                .into_iter()
                .map(|i| format!("-I{}", i.to_str().unwrap())),
        )
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    // let vendor_build_dir = out_dir.join("build-sofia-sip");
    // let project_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    // let vendor_dir = project_dir.join("vendor");
    // let vendor_configure_script = vendor_dir.join("configure");
    // fs::create_dir(&vendor_build_dir).ok();

    // /* FIXME: this will not work on windows */
    // Command::new("sh")
    //     .arg("-c")
    //     .arg(format!(
    //         "cd {} && ./bootstrap.sh",
    //         vendor_dir.to_str().unwrap()
    //     ))
    //     .output()
    //     .expect("failed to execute bootstrap.sh");

    // Command::new("sh")
    //     .arg("-c")
    //     .arg(format!(
    //         "cd {} && {}",
    //         vendor_build_dir.to_str().unwrap(),
    //         vendor_configure_script.to_str().unwrap()
    //     ))
    //     .output()
    //     .expect("failed to execute ./configure");

    // Command::new("sh")
    //     .arg("-c")
    //     .arg(&format!(
    //         "cd {} && make",
    //         vendor_build_dir.to_str().unwrap(),
    //     ))
    //     .output()
    //     .expect("failed to execute make");

    // Command::new("sh").arg("-c").arg(&!format!("cp {}/"))
    bindings
        .write_to_file(out_dir.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
