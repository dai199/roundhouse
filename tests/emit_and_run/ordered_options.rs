use super::ordered_options_app::{app, EXPECTED, SCRIPT};

#[test]
fn an_ordered_options_key_reads_through_its_name_on_ruby() {
    let run = app().run_ruby(SCRIPT);
    run.assert_passes();
    assert_eq!(run.stdout, EXPECTED);
}

/// A `delegate` whose names are read only as the program runs is no methods on Spinel, which defines no
/// `Module#delegate` and skips the call: reported there.
#[test]
fn a_delegate_the_expansion_cannot_read_is_reported_on_spinel() {
    let (_emitted, errors) = super::emit_and_run::real_blog()
        .write("app/lib/loose_box.rb", "class LooseBox\n  def self.names = [:a]\n\n  def target = nil\n\n  delegate(*names.dup, to: :target)\nend\n")
        .emit(roundhouse::project::BuildTarget::Spinel);
    assert!(errors.iter().any(|e| e.contains("Spinel defines no `Module#delegate`")), "{errors:#?}");
}
