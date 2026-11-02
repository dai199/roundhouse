//! The native half of `support/solid_queue_worker.rs`.

use super::solid_queue_worker_contract as solid_queue_worker;

#[test]
#[ignore = "requires the Spinel toolchain, run in its CI lane"]
fn solid_queue_worker_runs_natively() {
    let run = solid_queue_worker::app().run_spinel_with_db(solid_queue_worker::ASSERTIONS);
    run.assert_passes();
    assert!(run.stdout.contains("solid_queue worker passed"), "{}", run.stdout);
}
