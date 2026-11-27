//! `attr_reader *PROPERTIES` over a constant the class body holds as a literal list (mission_control's
//! JobsRelation): each name gets its reader, as Ruby defines them. Ingest dropped the splat, so `status` had no
//! reader and lobsters' Spinel tree answered its calls with NoMethodError.

use super::emit_and_run;

const PROBE: &str = r#"class Probe
  PROPERTIES = %i[queue_name status].freeze
  attr_reader(*PROPERTIES)

  def initialize
    @queue_name = "default"
    @status = :pending
  end

  def label
    queue_name + " " + status.to_s
  end
end
"#;

fn app() -> emit_and_run::Overlay {
    emit_and_run::real_blog()
        .edit("config/routes.rb", "  root \"articles#index\"\n", "  root \"articles#index\"\n  get \"/probe\", to: \"probes#index\"\n")
        .write("lib/probe.rb", PROBE)
        .write("app/controllers/probes_controller.rb", "class ProbesController < ApplicationController\n  def index\n    render plain: Probe.new.label\n  end\nend\n")
}

#[test]
fn a_splatted_constant_list_gets_its_readers() {
    let run = app().run_ruby(
        r##"response = Main.run_rack("REQUEST_METHOD" => "GET", "PATH_INFO" => "/probe", "QUERY_STRING" => "", "rack.input" => StringIO.new(""))
raise "GET /probe answered #{response[0]}" unless response[0] == 200
text = +""
response[2].each { |part| text << part }
raise "probe: #{text.inspect}" unless text == "default pending"
puts "attr splat passed"
"##,
    );
    run.assert_passes();
    let probe = std::fs::read_to_string(run.emitted.join("app/models/probe.rb")).expect("emitted probe");
    assert!(probe.contains("def status"), "{probe}");
}

#[test]
#[ignore = "requires the Spinel toolchain"]
fn a_splatted_constant_list_gets_its_readers_on_spinel() {
    let run = app().run_spinel(
        r##"controller = ProbesController.new
controller.request_method = "GET"
controller.params = {}
controller.process_action(:index)
raise "probe: #{controller.body}" unless controller.body == "default pending"
puts "attr splat passed"
"##,
    );
    run.assert_passes();
}
