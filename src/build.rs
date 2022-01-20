//! This module contains functions that can be used in a crate's `build.rs` file to execute parts of
//! the gettext workflow.

use std::path::PathBuf;
use std::process::Command;
use std::{env, ffi, fs};

/// Make sure the MO files in `./locales` are up-to-date and rerun build.rs if anything changed.
///
/// This generates new MO files for all existing PO files and tells the compiler to rerun the build
/// script if any PO file changed.
///
/// See [here](https://www.gnu.org/software/gettext/manual/gettext.html#Overview-of-GNU-gettext) for
/// more information on the `gettext` workflow.
pub fn update_mo_files() {
    const LOCALES_DIR: &str = "locales";

    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("failed to read OUT_DIR envvar"))
        .join(LOCALES_DIR);

    // Make sure the output directory exists.
    fs::create_dir_all(&out_dir).expect("failed to create locales output dir");

    for file in fs::read_dir(LOCALES_DIR).expect("failed to read locales directory") {
        let file = file.expect("failed to read po file");
        if file.path().extension() != Some(ffi::OsStr::new("po")) {
            continue;
        }

        // Get file names of po and mo files.
        let po_file_path = file.path();
        let mut mo_file_name: PathBuf = po_file_path
            .file_stem()
            .expect("failed to get po file name")
            .into();
        if !mo_file_name.set_extension("mo") {
            panic!("failed to set mo extension of locales file");
        }
        let output_file = out_dir.join(mo_file_name);

        // Use msgfmt to read the po files and create the mo files.
        let outcome = Command::new("msgfmt")
            .arg("--output-file")
            .arg(&output_file)
            .arg(&po_file_path)
            .output();
        match outcome {
            Ok(std::process::Output { status, .. }) if status.success() => (),
            outcome => panic!("failed to update MO files (is `msgfmt` in `PATH`?): {:#?}", outcome),
        };

        println!("cargo:rerun-if-changed={}", file.path().display());
    }
}
