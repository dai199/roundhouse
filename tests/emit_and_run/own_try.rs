//! A `try` whose class defines its own (parslet's atoms' `try(source, context, consume_all)`, called as
//! `obj.try(source, self, consume_all)` from its parse cache): ActiveSupport's Object#try is not the method, so the
//! call is not written as its body (`respond_to?(source)` raised on lobsters' search parse). An untyped receiver in a
//! program with such a class takes the class's `try` where it answers one, and ActiveSupport's otherwise.

use super::emit_and_run;

const PROBE: &str = r#"class Atom
  def try(source, context, consume_all)
    source + (consume_all ? "!" : "?")
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

  def untyped
    name = "label"
    JSON.parse(@raw)["word"].try(name) || "none"
  end

  def picked
    name = "label"
    pick = @raw.empty? ? Atom.new : Plain.new
    pick.try(name)
  end
end
"#;

fn app() -> emit_and_run::Overlay {
    emit_and_run::real_blog()
        .edit("config/routes.rb", "  root \"articles#index\"\n", "  root \"articles#index\"\n  get \"/probe\", to: \"probes#index\"\n")
        .write("lib/probe.rb", PROBE)
        .write(
            "app/controllers/probes_controller.rb",
            "class ProbesController < ApplicationController\n  def index\n    probe = Probe.new('{\"word\":\"hey\"}')\n    render plain: probe.typed + \" \" + probe.untyped + \" \" + probe.picked\n  end\nend\n",
        )
}

const EXPECTED: &str = "atom! none plain";

#[test]
fn a_class_s_own_try_is_called() {
    let run = app().run_ruby(&format!(
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
}

#[test]
#[ignore = "requires the Spinel toolchain"]
fn a_class_s_own_try_is_called_on_spinel() {
    let run = app().run_spinel(&format!(
        r##"controller = ProbesController.new
controller.request_method = "GET"
controller.params = {{}}
controller.process_action(:index)
raise "probe: #{{controller.body}}" unless controller.body == "{EXPECTED}"
puts "own try passed"
"##
    ));
    run.assert_passes();
}
