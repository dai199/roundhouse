use super::view_component_gem::{app, KNOWN_GAPS, SCRIPT};

#[test]
#[ignore = "requires the Spinel toolchain, run in its CI lane"]
fn a_component_renders_its_template_natively() {
    let Some(app) = app() else { return };
    let run = app.run_spinel_with_db(SCRIPT);
    let unknown: Vec<_> = run.errors.iter().filter(|e| !KNOWN_GAPS.iter().any(|gap| e.contains(gap))).collect();
    assert!(run.program_succeeded() && unknown.is_empty(), "{unknown:#?}\n{}\n{}", run.stdout, run.stderr);
    assert!(run.stdout.contains("view_component passed"), "{}", run.stdout);
}
