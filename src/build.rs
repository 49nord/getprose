//! This module contains functions that can be used in a crate's `build.rs` file to execute parts of
//! the gettext workflow.

use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::Command;
use std::{env, ffi, fs};
use typed_builder::TypedBuilder;
use walkdir::WalkDir;

/// TODO: Document
#[derive(TypedBuilder)]
pub struct XgettextArguments<'a> {
    /// Encoding of source files (default "UTF-8").
    #[builder(default = "UTF-8")]
    code_encoding: &'a str,
    /// Recognize the specified language (default "C").
    #[builder(default = "C")]
    language: &'a str,

    /// Name of the package. Ignored if [omit_header] is set.
    #[builder(default)]
    package_name: &'a str,
    /// Package version. Ignored if [omit_header] is set.
    #[builder(default)]
    package_version: &'a str,
    /// Copyright holder in output. Ignored if [omit_header] is set.
    #[builder(default)]
    copyright_holder: &'a str,
    /// Place comments preceding keyword lines starting with `comment_key` into the output file.
    #[builder(default, setter(strip_option, into))]
    comment_key: Option<String>,
    /// Omit output header lines. Might cause `xgettext` to omit non-ascii characters.
    #[builder(default)]
    omit_header: bool,
    /// Sort strings alphabetically.
    #[builder(default)]
    sort_output: bool,
    /// Do not include the locations of source strings.
    #[builder(default)]
    no_location: bool,
    /// Do not break long messages into several lines.
    #[builder(default)]
    no_wrap: bool,
    /// Create a POT file even if empty.
    #[builder(default)]
    force_pot: bool,
    /// Do not add the creation date to the header.
    #[builder(default)]
    no_creation_date: bool,

    /// Files which are searched for usage of `gettext`, `ngettext`, `pgettext` or `npgettext`.
    /// If `None` [create_pot_file] defaults to all `.rs` files in `./src`.
    #[builder(default, setter(strip_option))]
    input_files: Option<Vec<String>>,
}

/// Creates a gettext POT file at `output_file` by calling `xgettext` with `args` arguments.
pub fn create_pot_file(output_file: &str, args: XgettextArguments) {
    let mut cmd = Command::new("xgettext");

    // first add all used options to the command
    cmd.arg(format!("--from-code={}", args.code_encoding))
        .arg(format!("--language={}", args.language))
        .arg(format!("--package-name={}", args.package_name))
        .arg(format!("--package-version={}", args.package_version))
        .arg(format!("--copyright-holder={}", args.copyright_holder))
        .arg(format!("--output={}", output_file));

    add_arg_if(&mut cmd, "--omit-header", args.omit_header);
    add_arg_if(&mut cmd, "--sort-output", args.sort_output);
    add_arg_if(&mut cmd, "--no-location", args.no_location);
    add_arg_if(&mut cmd, "--no-wrap", args.no_wrap);
    add_arg_if(&mut cmd, "--force-po", args.force_pot);

    if let Some(comment) = args.comment_key {
        cmd.arg(format!("--add-comment={}", comment));
    }

    // If not files are given, get the paths to all `.rs` files in `src`.
    let input_files = match args.input_files {
        Some(files) => files,
        None => WalkDir::new("./src")
            .into_iter()
            .filter_map(|entry| match entry {
                Ok(entry) => Some(entry.path().to_string_lossy().to_string()),
                Err(_) => None,
            })
            .collect(),
    };
    cmd.args(&input_files);

    // Execute the command and report possible errors.
    let output = cmd.output().expect("could not execute xgettext");
    if !output.status.success() {
        panic!(
            "execution of xgettext failed with status: {}\n{}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        );
    }

    // If creation of the pot file was successful remove its `POT-Creation-Date` header.
    let output = std::path::Path::new(output_file);
    if args.no_creation_date && output.exists() {
        let file = fs::OpenOptions::new()
            .read(true)
            .open(output)
            .unwrap_or_else(|err| panic!("could not open \"{}\": {}", output_file, err));

        let lines = BufReader::new(file)
            .lines()
            .map(|line| line.expect("failed to read output file"))
            .filter(|line| !line.starts_with("\"POT-Creation-Date"))
            .collect::<Vec<_>>()
            .join("\n");

        fs::write(output, lines + "\n")
            .expect("could not write output file without POT-Creation-Date");
    }

    // No need to rerun the build script if no source file changed.
    for file in input_files {
        println!("cargo:rerun-if-changed={}", file);
    }
}

/// Add `flag_str` to `command` as an arguement if `flag` is `true`.
fn add_arg_if(command: &mut Command, flag_str: &str, flag: bool) {
    if flag {
        command.arg(flag_str);
    }
}

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
            outcome => panic!(
                "failed to update MO files (is `msgfmt` in `PATH`?): {:#?}",
                outcome
            ),
        };

        println!("cargo:rerun-if-changed={}", file.path().display());
    }
}
