use libtest2::RunError;
use libtest2::RunResult;
use libtest2::State;

libtest2::libtest2_main!(
    check_toph,
    check_katara,
    check_sokka,
    long_computation,
    compile_fail_dummy
);

// Tests

fn check_toph(_state: &State) -> RunResult {
    Ok(())
}
fn check_katara(_state: &State) -> RunResult {
    Ok(())
}
fn check_sokka(_state: &State) -> RunResult {
    Err(RunError::fail("Sokka tripped and fell :("))
}
fn long_computation(state: &State) -> RunResult {
    state.ignore_for("slow")?;

    std::thread::sleep(std::time::Duration::from_secs(1));
    Ok(())
}
fn compile_fail_dummy(_state: &State) -> RunResult {
    Ok(())
}
