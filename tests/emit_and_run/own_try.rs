//! A `try` whose class defines its own (parslet's atoms' `try(source, context, consume_all)`, called as
//! `obj.try(source, self, consume_all)` from its parse cache): ActiveSupport's Object#try is not the method, so the
//! call is not written as its body (`respond_to?(source)` raised on lobsters' search parse). An untyped receiver in a
//! program with such a class takes the class's `try` where it answers one, and ActiveSupport's otherwise. A receiver
//! only a subclass's method types (parslet's `Parser#try` calling the `root` each parser defines) takes its own.

use super::emit_and_run;

const PROBE: &str = r#"class Atom
  def try(source, context, consume_all)
    source + (consume_all ? "!" : "?")
  end
end

class Grammar
  def run(source)
    root.try(source, self, true)
  end
end

class WordGrammar < Grammar
  def initialize
    @atom = Atom.new
  end

  def root
    @atom
  end
end

class Plain
  def label
    "plain"
  end
end

class Probe
  def initialize(raw)
    @raw = raw
    @atom = Atom.new
  end

  def typed
    source = "atom"
    @atom.try(source, self, true)
  end

  # (untyped)

  def grammar
    source = "word"
    WordGrammar.new.run(source)
  end

  # (picked)
end
"#;

const UNTYPED: &str = "  def untyped\n    name = \"label\"\n    JSON.parse(@raw)[\"word\"].try(name) || \"none\"\n  end\n";
const PICKED: &str = "  def picked\n    name = \"label\"\n    pick = @raw.empty? ? Atom.new : Plain.new\n    pick.try(name)\n  end\n";

// The untyped and union receivers only where the tree runs on Ruby: on Spinel their ActiveSupport branch's
// `public_send` of a computed name does not compile (its answer is boxed into a slot the arms write unboxed), a gap
// this change leaves as it was.
fn app(all: bool) -> emit_and_run::Overlay {
    let probe = if all {
        PROBE.replace("  # (untyped)\n", UNTYPED).replace("  # (picked)\n", PICKED)
    } else {
        PROBE.replace("  # (untyped)\n", "").replace("  # (picked)\n", "")
    };
    let text = if all {
        "probe.typed + \" \" + probe.untyped + \" \" + probe.picked + \" \" + probe.grammar"
    } else {
        "probe.typed + \" \" + probe.grammar"
    };
    emit_and_run::real_blog()
        .edit("config/routes.rb", "  root \"articles#index\"\n", "  root \"articles#index\"\n  get \"/probe\", to: \"probes#index\"\n")
        .write("lib/probe.rb", &probe)
        .write(
            "app/controllers/probes_controller.rb",
            &format!("class ProbesController < ApplicationController\n  def index\n    probe = Probe.new('{{\"word\":\"hey\"}}')\n    render plain: {text}\n  end\nend\n"),
        )
}

const EXPECTED: &str = "atom! none plain word!";

#[test]
fn a_class_s_own_try_is_called() {
    let run = app(true).run_ruby(&format!(
        r##"response = Main.run_rack("REQUEST_METHOD" => "GET", "PATH_INFO" => "/probe", "QUERY_STRING" => "", "rack.input" => StringIO.new(""))
raise "GET /probe answered #{{response[0]}}" unless response[0] == 200
text = +""
response[2].each {{ |part| text << part }}
raise "probe: #{{text.inspect}}" unless text == "{EXPECTED}"
puts "own try passed"
"##
    ));
    run.assert_passes();
    let probe = std::fs::read_to_string(run.emitted.join("app/models/probe.rb")).expect("emitted probe");
    assert!(probe.contains("@atom.try(source, self, true)"), "{probe}");
    // A method only the subclasses define answers as they do, so `root` is an Atom or nil and its own `try` is called.
    assert!(!probe.contains("public_send(source"), "{probe}");
}

#[test]
#[ignore = "requires the Spinel toolchain"]
fn a_class_s_own_try_is_called_on_spinel() {
    let run = app(false).run_spinel(&format!(
        r##"controller = ProbesController.new
controller.request_method = "GET"
controller.params = {{}}
controller.process_action(:index)
raise "probe: #{{controller.body}}" unless controller.body == "atom! word!"
puts "own try passed"
"##
    ));
    run.assert_passes();
}
