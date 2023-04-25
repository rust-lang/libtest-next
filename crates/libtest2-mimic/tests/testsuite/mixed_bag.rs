fn test_cmd() -> snapbox::cmd::Command {
    static BIN: once_cell::sync::Lazy<(std::path::PathBuf, std::path::PathBuf)> =
        once_cell::sync::Lazy::new(|| {
            let package_root = crate::util::new_test(
                r#"
fn main() {
    use libtest2_mimic::Trial;
    use libtest2_mimic::RunError;
    libtest2_mimic::Harness::with_env()
        .cases(vec![
            Trial::test("cat", |_| Ok(())),
            Trial::test("dog", |_| Err(RunError::msg("was not a good boy"))),
            Trial::test("fox", |_| Ok(())),
            Trial::test("bunny", |state| {
                state.ignore_for("fails")?;
                Err(RunError::msg("jumped too high"))
            }),
            Trial::test("frog", |state| {
                state.ignore_for("slow")?;
                Ok(())
            }),
            Trial::test("owl", |state| {
                state.ignore_for("fails")?;
                Err(RunError::msg("broke neck"))
            }),
            Trial::test("fly", |state| {
                state.ignore_for("fails")?;
                Ok(())
            }),
            Trial::test("bear", |state| {
                state.ignore_for("fails")?;
                Err(RunError::msg("no honey"))
            }),
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
running 8 tests
test cat   ... ok
test dog   ... FAILED
test fox   ... ok
test bunny ... ignored
test frog  ... ignored
test owl   ... ignored
test fly   ... ignored
test bear  ... ignored

failures:

---- dog ----
was not a good boy


failures:
    dog

test result: FAILED. 2 passed; 1 failed; 5 ignored; 0 filtered out; finished in [..]s

"#,
        r#"
running 8 tests
test cat   ... ok
test dog   ... FAILED
test fox   ... ok
test bunny ... ignored
test frog  ... ignored
test owl   ... ignored
test fly   ... ignored
test bear  ... ignored

failures:

---- dog ----
was not a good boy


failures:
    dog

test result: FAILED. 2 passed; 1 failed; 5 ignored; 0 filtered out; finished in [..]s

"#,
    )
}

#[test]
fn test_mode() {
    check(
        &["--test"],
        101,
        r#"
running 8 tests
test cat   ... ok
test dog   ... FAILED
test fox   ... ok
test bunny ... ignored
test frog  ... ignored
test owl   ... ignored
test fly   ... ignored
test bear  ... ignored

failures:

---- dog ----
was not a good boy


failures:
    dog

test result: FAILED. 2 passed; 1 failed; 5 ignored; 0 filtered out; finished in 0.00s

"#,
        r#"
running 8 tests
...

failures:

---- dog ----
was not a good boy


failures:
    dog

test result: FAILED. 2 passed; 1 failed; 5 ignored; 0 filtered out; finished in 0.00s

"#,
    )
}

#[test]
fn bench_mode() {
    check(
        &["--bench"],
        101,
        r#"
running 8 tests
test cat   ... ok
test dog   ... FAILED
test fox   ... ok
test bunny ... ignored
test frog  ... ignored
test owl   ... ignored
test fly   ... ignored
test bear  ... ignored

failures:

---- dog ----
was not a good boy


failures:
    dog

test result: FAILED. 2 passed; 1 failed; 5 ignored; 0 filtered out; finished in 0.00s

"#,
        r#"
running 8 tests
...

failures:

---- dog ----
was not a good boy


failures:
    dog

test result: FAILED. 2 passed; 1 failed; 5 ignored; 0 filtered out; finished in 0.00s

"#,
    )
}

#[test]
fn list() {
    check(
        &["--list"],
        0,
        r#"cat: test
dog: test
fox: test
bunny: test
frog: test
owl: test
fly: test
bear: test

8 tests

"#,
        r#"cat: test
dog: test
fox: test
bunny: test
frog: test
owl: test
fly: test
bear: test

8 tests

"#,
    )
}

#[test]
fn list_ignored() {
    check(
        &["--list", "--ignored"],
        0,
        r#"cat: test
dog: test
fox: test
bunny: test
frog: test
owl: test
fly: test
bear: test

8 tests

"#,
        r#"cat: test
dog: test
fox: test
bunny: test
frog: test
owl: test
fly: test
bear: test

8 tests

"#,
    );
}

#[test]
fn list_with_filter() {
    check(
        &["--list", "a"],
        0,
        r#"cat: test
bear: test

2 tests

"#,
        r#"cat: test
bear: test

2 tests

"#,
    );
}

#[test]
fn filter_c() {
    check(
        &["a"],
        0,
        r#"
running 2 tests
test cat  ... ok
test bear ... ignored

test result: ok. 1 passed; 0 failed; 1 ignored; 6 filtered out; finished in [..]s

"#,
        r#"
running 2 tests
...

test result: ok. 1 passed; 0 failed; 1 ignored; 6 filtered out; finished in [..]s

"#,
    )
}

#[test]
fn filter_o_test() {
    check(
        &["--test", "a"],
        0,
        r#"
running 2 tests
test cat  ... ok
test bear ... ignored

test result: ok. 1 passed; 0 failed; 1 ignored; 6 filtered out; finished in [..]s

"#,
        r#"
running 2 tests
...

test result: ok. 1 passed; 0 failed; 1 ignored; 6 filtered out; finished in [..]s

"#,
    );
}

#[test]
fn filter_o_test_include_ignored() {
    check(
        &["--test", "--include-ignored", "o"],
        101,
        r#"
running 4 tests
test dog  ... FAILED
test fox  ... ok
test frog ... ok
test owl  ... FAILED

failures:

---- dog ----
was not a good boy

---- owl ----
broke neck


failures:
    dog
    owl

test result: FAILED. 2 passed; 2 failed; 0 ignored; 4 filtered out; finished in [..]s

"#,
        r#"
running 4 tests
...

failures:

---- dog ----
was not a good boy

---- owl ----
broke neck


failures:
    dog
    owl

test result: FAILED. 2 passed; 2 failed; 0 ignored; 4 filtered out; finished in [..]s

"#,
    )
}

#[test]
fn filter_o_test_ignored() {
    check(
        &["--test", "--ignored", "o"],
        101,
        r#"
running 4 tests
test dog  ... FAILED
test fox  ... ok
test frog ... ok
test owl  ... FAILED

failures:

---- dog ----
was not a good boy

---- owl ----
broke neck


failures:
    dog
    owl

test result: FAILED. 2 passed; 2 failed; 0 ignored; 4 filtered out; finished in [..]s

"#,
        r#"
running 4 tests
...

failures:

---- dog ----
was not a good boy

---- owl ----
broke neck


failures:
    dog
    owl

test result: FAILED. 2 passed; 2 failed; 0 ignored; 4 filtered out; finished in [..]s

"#,
    )
}

#[test]
fn normal_include_ignored() {
    check(
        &["--include-ignored"],
        101,
        r#"
running 8 tests
test cat   ... ok
test dog   ... FAILED
test fox   ... ok
test bunny ... FAILED
test frog  ... ok
test owl   ... FAILED
test fly   ... ok
test bear  ... FAILED

failures:

---- bear ----
no honey

---- bunny ----
jumped too high

---- dog ----
was not a good boy

---- owl ----
broke neck


failures:
    bear
    bunny
    dog
    owl

test result: FAILED. 4 passed; 4 failed; 0 ignored; 0 filtered out; finished in 0.00s

"#,
        r#"
running 8 tests
...

failures:

---- bear ----
no honey

---- bunny ----
jumped too high

---- dog ----
was not a good boy

---- owl ----
broke neck


failures:
    bear
    bunny
    dog
    owl

test result: FAILED. 4 passed; 4 failed; 0 ignored; 0 filtered out; finished in 0.00s

"#,
    )
}

#[test]
fn normal_ignored() {
    check(
        &["--ignored"],
        101,
        r#"
running 8 tests
test cat   ... ok
test dog   ... FAILED
test fox   ... ok
test bunny ... FAILED
test frog  ... ok
test owl   ... FAILED
test fly   ... ok
test bear  ... FAILED

failures:

---- bear ----
no honey

---- bunny ----
jumped too high

---- dog ----
was not a good boy

---- owl ----
broke neck


failures:
    bear
    bunny
    dog
    owl

test result: FAILED. 4 passed; 4 failed; 0 ignored; 0 filtered out; finished in 0.00s

"#,
        r#"
running 8 tests
...

failures:

---- bear ----
no honey

---- bunny ----
jumped too high

---- dog ----
was not a good boy

---- owl ----
broke neck


failures:
    bear
    bunny
    dog
    owl

test result: FAILED. 4 passed; 4 failed; 0 ignored; 0 filtered out; finished in 0.00s

"#,
    )
}

#[test]
fn lots_of_flags() {
    check(
        &["--ignored", "--skip", "g", "--test", "o"],
        101,
        r#"
running 2 tests
test fox ... ok
test owl ... FAILED

failures:

---- owl ----
broke neck


failures:
    owl

test result: FAILED. 1 passed; 1 failed; 0 ignored; 6 filtered out; finished in [..]s

"#,
        r#"
running 2 tests
...

failures:

---- owl ----
broke neck


failures:
    owl

test result: FAILED. 1 passed; 1 failed; 0 ignored; 6 filtered out; finished in [..]s

"#,
    )
}

#[test]
fn terse_output() {
    check(
        &["--quiet"],
        101,
        r#"
running 8 tests
.F.iiiii
failures:

---- dog ----
was not a good boy


failures:
    dog

test result: FAILED. 2 passed; 1 failed; 5 ignored; 0 filtered out; finished in [..]s

"#,
        r#"
running 8 tests
...
failures:

---- dog ----
was not a good boy


failures:
    dog

test result: FAILED. 2 passed; 1 failed; 5 ignored; 0 filtered out; finished in [..]s

"#,
    )
}
