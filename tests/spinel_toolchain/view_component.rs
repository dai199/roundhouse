use super::view_component_gem::{app, KNOWN_GAPS, SCRIPT};

#[test]
#[ignore = "requires the Spinel toolchain, run in its CI lane"]
fn a_component_renders_its_template_natively() {
    let Some(app) = app() else { return };
    let run = app.run_spinel_with_db(SCRIPT);
    let unknown: Vec<_> = run.errors.iter().filter(|e| !KNOWN_GAPS.iter().any(|gap| e.contains(gap))).collect();
    assert!(unknown.is_empty(), "{unknown:#?}");
    run.assert_passes();
    assert!(run.stdout.contains("view_component passed"), "{}", run.stdout);
}
