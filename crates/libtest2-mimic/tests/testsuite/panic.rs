fn test_cmd() -> snapbox::cmd::Command {
    static BIN: once_cell::sync::Lazy<(std::path::PathBuf, std::path::PathBuf)> =
        once_cell::sync::Lazy::new(|| {
            let package_root = crate::util::new_test(
                r#"
fn main() {
    use libtest2_mimic::Trial;
    libtest2_mimic::Harness::with_env()
        .cases(vec![
            Trial::test("passes", |_| Ok(())),
            Trial::test("panics", |_| panic!("uh oh")),
        ])
        .main();
}
"#,
                false,
            );
            let bin = crate::util::compile_test(&package_root);
            (bin, package_root)
        });
    snapbox::cmd::Command::new(&BIN.0).current_dir(&BIN.1)
}

fn check(args: &[&str], code: i32, single: &str, parallel: &str) {
    test_cmd()
        .args(args)
        .args(["--test-threads", "1"])
        .assert()
        .code(code)
        .stdout_matches(single);
    test_cmd()
        .args(args)
        .assert()
        .code(code)
        .stdout_matches(parallel);
}

#[test]
fn normal() {
    check(
        &[],
        101,
        r#"
running 2 tests
test panics ... FAILED
test passes ... ok

failures:

---- panics ----
test panicked: uh oh


failures:
    panics

test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 filtered out; finished in [..]s

"#,
        r#"
running 2 tests
...

failures:

---- panics ----
test panicked: uh oh


failures:
    panics

test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 filtered out; finished in [..]s

"#,
    );
}
