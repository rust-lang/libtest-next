pub fn new_test(test: &str, harness: bool) -> std::path::PathBuf {
    static SUFFIX: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
    let suffix = SUFFIX.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let target_name = format!("t{suffix}");

    let package_root = tempdir().join("libtest2_mimic").join(&target_name);

    let mimic_relpath = mimic_relpath(&package_root);
    let mimic_relpath = mimic_relpath.display();

    std::fs::create_dir_all(&package_root).unwrap();
    std::fs::write(
        package_root.join("Cargo.toml"),
        format!(
            r#"
[workspace]

[package]
name = "t{suffix}"
version = "0.0.0"
edition = "2021"

[lib]
path = "lib.rs"

[dev-dependencies]
libtest2-mimic.path = "{mimic_relpath}"

[[test]]
name = "{target_name}"
path = "test.rs"
harness = {harness}
"#
        ),
    )
    .unwrap();
    std::fs::write(package_root.join("lib.rs"), "").unwrap();
    std::fs::write(package_root.join("test.rs"), test).unwrap();

    package_root
}

pub fn compile_test(package_root: &std::path::PathBuf) -> std::path::PathBuf {
    let manifest_path = package_root.join("Cargo.toml");
    let target_name = package_root.file_name().unwrap().to_str().unwrap();
    let args = [
        std::ffi::OsString::from("--target-dir"),
        target_dir().into_os_string(),
    ];
    tests::compile_test(&manifest_path, target_name, args)
}

fn mimic_relpath(root: &std::path::Path) -> std::path::PathBuf {
    let current_dir = std::env::current_dir().unwrap();
    let relpath = pathdiff::diff_paths(&current_dir, &root).unwrap();
    let normalized = relpath.as_os_str().to_str().unwrap().replace('\\', "/");
    std::path::PathBuf::from(normalized)
}

fn target_dir() -> std::path::PathBuf {
    tempdir().join("libtest2_mimic_target")
}

fn tempdir() -> std::path::PathBuf {
    const TEMPDIR: &str = env!("CARGO_TARGET_TMPDIR");

    let tempdir = std::path::Path::new(TEMPDIR);
    std::fs::create_dir_all(tempdir).unwrap();
    dunce::canonicalize(&tempdir).unwrap()
}

mod tests {
    pub fn compile_test<'a>(
        manifest_path: &std::path::Path,
        target_name: &str,
        args: impl IntoIterator<Item = impl AsRef<std::ffi::OsStr>>,
    ) -> std::path::PathBuf {
        let messages = escargot::CargoBuild::new()
            .current_target()
            .current_release()
            .manifest_path(manifest_path)
            .env("CARGO_TERM_COLOR", "always")
            .test(target_name)
            .args(args)
            .exec()
            .unwrap_or_else(|e| panic!("{e}"));
        for message in messages {
            let message = message.unwrap_or_else(|e| panic!("{e}"));
            let message = message.decode().unwrap_or_else(|e| panic!("{e}"));
            if let Some((name, bin)) = decode_test_message(&message) {
                assert_eq!(target_name, name);
                return bin;
            }
        }

        panic!("Unknown error building test {}", target_name)
    }

    #[allow(clippy::type_complexity)]
    fn decode_test_message<'m>(
        message: &'m escargot::format::Message,
    ) -> Option<(&'m str, std::path::PathBuf)> {
        match message {
            escargot::format::Message::CompilerMessage(msg) => {
                let level = msg.message.level;
                if level == escargot::format::diagnostic::DiagnosticLevel::Ice
                    || level == escargot::format::diagnostic::DiagnosticLevel::Error
                {
                    let output = msg
                        .message
                        .rendered
                        .as_deref()
                        .unwrap_or_else(|| msg.message.message.as_ref())
                        .to_owned();
                    panic!("{output}");
                } else {
                    None
                }
            }
            escargot::format::Message::CompilerArtifact(artifact) => {
                if artifact.profile.test && is_test_target(&artifact.target) {
                    let path = artifact
                        .executable
                        .clone()
                        .expect("cargo is new enough for this to be present");
                    let bin = path.into_owned();
                    Some((artifact.target.name.as_ref(), bin))
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn is_test_target(target: &escargot::format::Target) -> bool {
        target.crate_types == ["bin"] && target.kind == ["test"]
    }
}
