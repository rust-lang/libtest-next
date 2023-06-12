fn test_cmd() -> snapbox::cmd::Command {
    static BIN: once_cell::sync::Lazy<(std::path::PathBuf, std::path::PathBuf)> =
        once_cell::sync::Lazy::new(|| {
            let package_root = crate::util::new_test(
                r#"
libtest2::libtest2_main!(cat, dog, fox, bunny, frog, owl, fly, bear);

fn cat(_state: &libtest2::State) -> libtest2::RunResult {
    Ok(())
}

fn dog(_state: &libtest2::State) -> libtest2::RunResult {
    Err(libtest2::RunError::fail("was not a good boy"))
}

fn fox(_state: &libtest2::State) -> libtest2::RunResult {
    Ok(())
}

fn bunny(state: &libtest2::State) -> libtest2::RunResult {
    state.ignore_for("fails")?;
    Err(libtest2::RunError::fail("jumped too high"))
}

fn frog(state: &libtest2::State) -> libtest2::RunResult {
    state.ignore_for("slow")?;
    Ok(())
}

fn owl(state: &libtest2::State) -> libtest2::RunResult {
    state.ignore_for("fails")?;
    Err(libtest2::RunError::fail("broke neck"))
}

fn fly(state: &libtest2::State) -> libtest2::RunResult {
    state.ignore_for("fails")?;
    Ok(())
}

fn bear(state: &libtest2::State) -> libtest2::RunResult {
    state.ignore_for("fails")?;
    Err(libtest2::RunError::fail("no honey"))
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
test bear  ... ignored
test bunny ... ignored
test cat   ... ok
test dog   ... FAILED
test fly   ... ignored
test fox   ... ok
test frog  ... ignored
test owl   ... ignored

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

#[test]
fn test_mode() {
    check(
        &["--test"],
        101,
        r#"
running 8 tests
test bear  ... ignored
test bunny ... ignored
test cat   ... ok
test dog   ... FAILED
test fly   ... ignored
test fox   ... ok
test frog  ... ignored
test owl   ... ignored

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

#[test]
fn bench_mode() {
    check(
        &["--bench"],
        101,
        r#"
running 8 tests
test bear  ... ignored
test bunny ... ignored
test cat   ... ok
test dog   ... FAILED
test fly   ... ignored
test fox   ... ok
test frog  ... ignored
test owl   ... ignored

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

#[test]
fn list() {
    check(
        &["--list"],
        0,
        r#"bear: test
bunny: test
cat: test
dog: test
fly: test
fox: test
frog: test
owl: test

8 tests

"#,
        r#"bear: test
bunny: test
cat: test
dog: test
fly: test
fox: test
frog: test
owl: test

8 tests

"#,
    )
}

#[test]
fn list_ignored() {
    check(
        &["--list", "--ignored"],
        0,
        r#"bear: test
bunny: test
cat: test
dog: test
fly: test
fox: test
frog: test
owl: test

8 tests

"#,
        r#"bear: test
bunny: test
cat: test
dog: test
fly: test
fox: test
frog: test
owl: test

8 tests

"#,
    );
}

#[test]
fn list_with_filter() {
    check(
        &["--list", "a"],
        0,
        r#"bear: test
cat: test

2 tests

"#,
        r#"bear: test
cat: test

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
test bear ... ignored
test cat  ... ok

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
test bear ... ignored
test cat  ... ok

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
test bear  ... FAILED
test bunny ... FAILED
test cat   ... ok
test dog   ... FAILED
test fly   ... ok
test fox   ... ok
test frog  ... ok
test owl   ... FAILED

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

test result: FAILED. 4 passed; 4 failed; 0 ignored; 0 filtered out; finished in [..]s

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

test result: FAILED. 4 passed; 4 failed; 0 ignored; 0 filtered out; finished in [..]s

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
test bear  ... FAILED
test bunny ... FAILED
test cat   ... ok
test dog   ... FAILED
test fly   ... ok
test fox   ... ok
test frog  ... ok
test owl   ... FAILED

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

test result: FAILED. 4 passed; 4 failed; 0 ignored; 0 filtered out; finished in [..]s

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

test result: FAILED. 4 passed; 4 failed; 0 ignored; 0 filtered out; finished in [..]s

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
#[cfg(feature = "json")]
fn list_json() {
    check(
        &["-Zunstable-options", "--format=json", "--list", "a"],
        0,
        r#"{"event":"discover-start"}
{"event":"discover-case","name":"bear","mode":"test","run":true}
{"event":"discover-case","name":"bunny","mode":"test","run":false}
{"event":"discover-case","name":"cat","mode":"test","run":true}
{"event":"discover-case","name":"dog","mode":"test","run":false}
{"event":"discover-case","name":"fly","mode":"test","run":false}
{"event":"discover-case","name":"fox","mode":"test","run":false}
{"event":"discover-case","name":"frog","mode":"test","run":false}
{"event":"discover-case","name":"owl","mode":"test","run":false}
{"event":"discover-complete","elapsed_s":"[..]","seed":null}
"#,
        r#"{"event":"discover-start"}
{"event":"discover-case","name":"bear","mode":"test","run":true}
{"event":"discover-case","name":"bunny","mode":"test","run":false}
{"event":"discover-case","name":"cat","mode":"test","run":true}
{"event":"discover-case","name":"dog","mode":"test","run":false}
{"event":"discover-case","name":"fly","mode":"test","run":false}
{"event":"discover-case","name":"fox","mode":"test","run":false}
{"event":"discover-case","name":"frog","mode":"test","run":false}
{"event":"discover-case","name":"owl","mode":"test","run":false}
{"event":"discover-complete","elapsed_s":"[..]","seed":null}
"#,
    )
}

#[test]
#[cfg(feature = "json")]
fn test_json() {
    check(
        &["-Zunstable-options", "--format=json", "a"],
        0,
        r#"{"event":"discover-start"}
{"event":"discover-case","name":"bear","mode":"test","run":true}
{"event":"discover-case","name":"bunny","mode":"test","run":false}
{"event":"discover-case","name":"cat","mode":"test","run":true}
{"event":"discover-case","name":"dog","mode":"test","run":false}
{"event":"discover-case","name":"fly","mode":"test","run":false}
{"event":"discover-case","name":"fox","mode":"test","run":false}
{"event":"discover-case","name":"frog","mode":"test","run":false}
{"event":"discover-case","name":"owl","mode":"test","run":false}
{"event":"discover-complete","elapsed_s":"[..]","seed":null}
{"event":"suite-start"}
{"event":"case-start","name":"bear"}
{"event":"case-complete","name":"bear","mode":"test","status":"ignored","message":"fails","elapsed_s":"[..]"}
{"event":"case-start","name":"cat"}
{"event":"case-complete","name":"cat","mode":"test","status":null,"message":null,"elapsed_s":"[..]"}
{"event":"suite-complete","elapsed_s":"[..]"}
"#,
        r#"{"event":"discover-start"}
{"event":"discover-case","name":"bear","mode":"test","run":true}
{"event":"discover-case","name":"bunny","mode":"test","run":false}
{"event":"discover-case","name":"cat","mode":"test","run":true}
{"event":"discover-case","name":"dog","mode":"test","run":false}
{"event":"discover-case","name":"fly","mode":"test","run":false}
{"event":"discover-case","name":"fox","mode":"test","run":false}
{"event":"discover-case","name":"frog","mode":"test","run":false}
{"event":"discover-case","name":"owl","mode":"test","run":false}
{"event":"discover-complete","elapsed_s":"[..]","seed":null}
{"event":"suite-start"}
[..]
[..]
[..]
[..]
{"event":"suite-complete","elapsed_s":"[..]"}
"#,
    )
}

#[test]
#[cfg(feature = "junit")]
fn list_junit() {
    check(
        &["-Zunstable-options", "--format=junit", "--list", "a"],
        0,
        r#"bear: test
cat: test

2 tests

"#,
        r#"bear: test
cat: test

2 tests

"#,
    )
}

#[test]
#[cfg(feature = "junit")]
fn test_junit() {
    check(
        &["-Zunstable-options", "--format=junit", "a"],
        0,
        r#"<?xml version="1.0" encoding="UTF-8"?>
<testsuites>
<testsuite name="test" package="test" id="0" tests="2" errors="0" failures="0" skipped="1" >
<testcase classname="crate" name="cat" time="0.000s"/>
<system-out/>
<system-err/>
</testsuite>
</testsuites>
"#,
        r#"<?xml version="1.0" encoding="UTF-8"?>
<testsuites>
<testsuite name="test" package="test" id="0" tests="2" errors="0" failures="0" skipped="1" >
<testcase classname="crate" name="cat" time="0.000s"/>
<system-out/>
<system-err/>
</testsuite>
</testsuites>
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
ii.Fi.ii
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

#[test]
fn shuffle() {
    check(
        &["-Zunstable-options", "--list", "--shuffle-seed=1"],
        0,
        r#"fox: test
cat: test
fly: test
bear: test
owl: test
frog: test
bunny: test
dog: test

8 tests

"#,
        r#"fox: test
cat: test
fly: test
bear: test
owl: test
frog: test
bunny: test
dog: test

8 tests

"#,
    );
    check(
        &["-Zunstable-options", "--list", "--shuffle-seed=2"],
        0,
        r#"owl: test
dog: test
fox: test
frog: test
bear: test
fly: test
bunny: test
cat: test

8 tests

"#,
        r#"owl: test
dog: test
fox: test
frog: test
bear: test
fly: test
bunny: test
cat: test

8 tests

"#,
    );
}
