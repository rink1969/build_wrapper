use std::path::Path;
use std::process::{Command, exit};

/// If a command-line option matches `find_arg`, then apply the predicate `pred` on its value. If
/// true, then return it. The parameter is assumed to be either `--arg=value` or `--arg value`.
fn arg_value<'a>(
    args: impl IntoIterator<Item = &'a String>,
    find_arg: &str,
    pred: impl Fn(&str) -> bool,
) -> Option<&'a str> {
    let mut args = args.into_iter().map(String::as_str);

    while let Some(arg) = args.next() {
        let arg: Vec<_> = arg.splitn(2, '=').collect();
        if arg.get(0) != Some(&find_arg) {
            continue;
        }

        let value = arg.get(1).cloned().or_else(|| args.next());
        if value.as_ref().map_or(false, |p| pred(p)) {
            return value;
        }
    }
    None
}

/*
wrapper get args from Cargo like this:
args ["/home/rink/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin/clippy-driver",
"--edition=2018", "--crate-name", "clippytest", "examples/clippytest.rs", "--color", "always",
"--crate-type", "bin", "--emit=dep-info,metadata", "-C", "debuginfo=2",
"-C", "metadata=cb06b6e6fddc6ad6", "-C", "extra-filename=-cb06b6e6fddc6ad6",
"--out-dir", "/home/rink/work/rust_examples/target/debug/examples",
"-C", "incremental=/home/rink/work/rust_examples/target/debug/incremental",
"-L", "dependency=/home/rink/work/rust_examples/target/debug/deps",
"--sysroot", "/home/rink/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu",
"--cfg", "feature=\"cargo-clippy\""]
*/

pub fn main() {
    use std::env;

    let mut orig_args: Vec<String> = env::args().collect();

    // Setting RUSTC_WRAPPER causes Cargo to pass 'rustc' as the first argument.
    // We're invoking the compiler programmatically, so we ignore this/
    if orig_args.len() <= 1 {
        std::process::exit(1);
    }

    // Get the sysroot, looking from most specific to this invocation to the least:
    // - command line
    // - runtime environment
    //    - SYSROOT
    //    - RUSTUP_HOME, MULTIRUST_HOME, RUSTUP_TOOLCHAIN, MULTIRUST_TOOLCHAIN
    // - sysroot from rustc in the path
    // - compile-time environment
    let sys_root_arg = arg_value(&orig_args, "--sysroot", |_| true);
    let have_sys_root_arg = sys_root_arg.is_some();
    let sys_root = sys_root_arg
        .map(std::string::ToString::to_string)
        .or_else(|| std::env::var("SYSROOT").ok())
        .or_else(|| {
            let home = option_env!("RUSTUP_HOME").or(option_env!("MULTIRUST_HOME"));
            let toolchain = option_env!("RUSTUP_TOOLCHAIN").or(option_env!("MULTIRUST_TOOLCHAIN"));
            home.and_then(|home| toolchain.map(|toolchain| format!("{}/toolchains/{}", home, toolchain)))
        })
        .or_else(|| {
            Command::new("rustc")
                .arg("--print")
                .arg("sysroot")
                .output()
                .ok()
                .and_then(|out| String::from_utf8(out.stdout).ok())
                .map(|s| s.trim().to_owned())
        })
        .or_else(|| option_env!("SYSROOT").map(String::from))
        .expect("need to specify SYSROOT env var during clippy compilation, or use rustup or multirust");

    // Setting RUSTC_WRAPPER causes Cargo to pass 'rustc' as the first argument.
    // We're invoking the compiler programmatically, so we ignore this/
    if orig_args.len() <= 1 {
        std::process::exit(1);
    }
    if Path::new(&orig_args[1]).file_stem() == Some("rustc".as_ref()) {
        // we still want to be able to invoke it normally though
        orig_args.remove(1);
    }
    // this conditional check for the --sysroot flag is there so users can call
    // `clippy_driver` directly
    // without having to pass --sysroot or anything
    let mut args: Vec<String> = if have_sys_root_arg {
        orig_args.clone()
    } else {
        orig_args
            .clone()
            .into_iter()
            .chain(Some("--sysroot".to_owned()))
            .chain(Some(sys_root))
            .collect()
    };

    // process args
    let is_debug = arg_value(&orig_args, "debuginfo", |_| true).is_some();
    if !is_debug {
        // add line table in release version
        let mut cita_count = 0;
        for s in &orig_args {
            if s.contains("cita") {
                cita_count += 1;
            }
        }
        if cita_count > 2 {
            args.extend_from_slice(&["-C".to_owned(), "debuginfo=1".to_owned()]);
        }
    }

    // remove wrapper it self
    args.remove(0);

    let exit_status = std::process::Command::new("rustc")
        .args(&args)
        .status()
        .expect("could not run rustc");

    if exit_status.success() {
        exit(0);
    } else {
        exit(-1);
    }
}