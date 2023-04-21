use libtest2_harness::RunError;
use libtest2_harness::RunResult;
use libtest2_harness::SimpleCase;
use libtest2_harness::State;

fn main() {
    libtest2_harness::Harness::new()
        .case(SimpleCase::test("check_toph", check_toph))
        .case(SimpleCase::test("check_katara", check_katara))
        .case(SimpleCase::test("check_sokka", check_sokka))
        .case(SimpleCase::test("long_computation", long_computation))
        .case(SimpleCase::test("compile_fail_dummy", compile_fail_dummy))
        .main();
}

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
