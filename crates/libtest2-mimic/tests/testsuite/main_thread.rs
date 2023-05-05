#[test]
fn check_test_on_main_thread() {
    let package_root = crate::util::new_test(
        r#"
fn main() {
    use libtest2_mimic::Trial;
    let outer_thread = std::thread::current().id();
    libtest2_mimic::Harness::with_env()
        .cases(vec![
            Trial::test("check", move |_| {
                assert_eq!(outer_thread, std::thread::current().id());
                Ok(())
            })
        ])
        .main();
}
"#,
        false,
    );
    let bin = crate::util::compile_test(&package_root);
    snapbox::cmd::Command::new(bin)
        .current_dir(package_root)
        .assert()
        .success()
        .stdout_matches(
            "
running 1 test
...",
        );
}
