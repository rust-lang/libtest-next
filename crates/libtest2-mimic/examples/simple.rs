use libtest2_mimic::RunError;
use libtest2_mimic::RunResult;
use libtest2_mimic::State;
use libtest2_mimic::Trial;

fn main() {
    libtest2_mimic::Harness::with_env()
        .case(Trial::test("check_toph", check_toph))
        .case(Trial::test("check_katara", check_katara))
        .case(Trial::test("check_sokka", check_sokka))
        .case(Trial::test("long_computation", long_computation))
        .case(Trial::test("compile_fail_dummy", compile_fail_dummy))
        .main();
}

// Tests

fn check_toph(_state: &State) -> RunResult {
    Ok(())
}
fn check_katara(_state: &State) -> RunResult {
    Ok(())
}
fn check_sokka(_state: &State) -> RunResult {
    Err(RunError::msg("Sokka tripped and fell :("))
}
fn long_computation(state: &State) -> RunResult {
    state.ignore_for("slow")?;

    std::thread::sleep(std::time::Duration::from_secs(1));
    Ok(())
}
fn compile_fail_dummy(_state: &State) -> RunResult {
    Ok(())
}
