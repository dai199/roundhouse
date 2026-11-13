use super::view_component_gem::{app, KNOWN_GAPS, SCRIPT};

#[test]
fn a_component_renders_its_template_on_ruby() {
    let Some(app) = app() else { return };
    let run = app.run_ruby(SCRIPT);
    let unknown: Vec<_> = run
        .errors
        .iter()
        .filter(|e| !KNOWN_GAPS.iter().any(|gap| e.contains(gap)))
        .collect();
    assert!(unknown.is_empty(), "{unknown:#?}");
    for gap in KNOWN_GAPS {
        assert!(run.errors.iter().any(|e| e.contains(gap)), "{gap} is answered now: drop it from KNOWN_GAPS\n{:#?}", run.errors);
    }
    assert!(run.program_succeeded(), "{}\n{}", run.stdout, run.stderr);
    assert!(run.stdout.contains("view_component passed"), "{}", run.stdout);
}

