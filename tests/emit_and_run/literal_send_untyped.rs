//! A `send` of a literal name on a receiver of unknown class (fugit's `TimeCursor`, unrolled from
//! `define_method(k) { @t.send(k) }`): the Spinel tree resolved `send` to `Faye::WebSocket#send`, a class's own
//! `send`, and refused `wday_in_month` as returning a bool. The send is written `__send__`, Kernel's alone, which is
//! the method Ruby's `send` is for such a receiver. So is a typed one whose class has no `send` of its own (fugit's
//! `@t` is an `EoTime`, which the native build did not keep); a class with its own `send`, or a singleton one on a
//! constant receiver, keeps it, as does a union
//! holding such a class, and so does a String argument (a socket's own `send(data)`).

use super::emit_and_run;

const PROBE: &str = r#"class Clock
  def hour
    7
  end
end

class Watch
  def hour
    8
  end
end

class Channel
  def send(name)
    "sent #{name}"
  end
end

class Relay
  def self.send(name)
    "relayed #{name}"
  end

  def self.via_self
    self.send(:hour)
  end

  def self.via_local
    k = Relay
    k.send(:hour)
  end
end

class Probe
  CHANNEL = Channel.new

  def initialize(raw)
    @raw = raw
    @clock = Clock.new
    @channel = Channel.new
  end

  def hour
    @clock.send(:hour)
  end

  def relay
    @channel.send(:hour) + " " + CHANNEL.send(:hour)
  end

  def relayed
    Relay.send(:hour) + " " + Relay.via_self + " " + Relay.via_local
  end

  def either
    pick = @raw.empty? ? Watch.new : Clock.new
    mixed = @raw.empty? ? Clock.new : Channel.new
    pick.send(:hour).to_s + " " + mixed.send(:hour)
  end

  def shout
    JSON.parse(@raw)["word"].send(:upcase)
  end

  def typed
    "quiet".send(:upcase)
  end

  def named
    JSON.parse(@raw)["word"].send("capitalize")
  end
end
"#;

fn app() -> emit_and_run::Overlay {
    emit_and_run::real_blog()
        .edit("config/routes.rb", "  root \"articles#index\"\n", "  root \"articles#index\"\n  get \"/probe\", to: \"probes#index\"\n")
        .write("lib/probe.rb", PROBE)
        .write(
            "app/controllers/probes_controller.rb",
            "class ProbesController < ApplicationController\n  def index\n    probe = Probe.new('{\"word\":\"hey\"}')\n    render plain: \"#{probe.shout} #{probe.typed} #{probe.named} #{probe.hour} #{probe.relay} #{probe.relayed} #{probe.either}\"\n  end\nend\n",
        )
}

const SCRIPT: &str = r##"def probe_text
  response = Main.run_rack("REQUEST_METHOD" => "GET", "PATH_INFO" => "/probe", "QUERY_STRING" => "", "rack.input" => StringIO.new(""))
  raise "GET /probe answered #{response[0]}" unless response[0] == 200
  text = +""
  response[2].each { |part| text << part }
  text
end
text = probe_text
raise "probe: #{text.inspect}" unless text == "HEY QUIET Hey 7 sent hour sent hour relayed hour relayed hour relayed hour 7 sent hour"
puts "literal send passed"
"##;

#[test]
fn a_literal_send_on_an_unknown_receiver_is_kernel_s() {
    let run = app().run_ruby(SCRIPT);
    run.assert_passes();
    let probe = std::fs::read_to_string(run.emitted.join("app/models/probe.rb")).expect("emitted probe");
    assert!(probe.contains(r#"["word"].__send__(:upcase)"#), "{probe}");
    assert!(probe.contains(r#""quiet".__send__(:upcase)"#), "{probe}");
    assert!(probe.contains("@clock.__send__(:hour)"), "{probe}");
    assert!(probe.contains("@channel.send(:hour)"), "{probe}");
    assert!(probe.contains("Relay.send(:hour)"), "{probe}");
    assert!(probe.contains("pick.__send__(:hour)"), "{probe}");
    assert!(probe.contains("mixed.send(:hour)"), "{probe}");
    assert!(probe.contains(r#"["word"].send("capitalize")"#), "{probe}");
}

#[test]
#[ignore = "requires the Spinel toolchain"]
fn a_literal_send_on_an_unknown_receiver_is_kernel_s_on_spinel() {
    let run = app().run_spinel(
        r##"controller = ProbesController.new
controller.request_method = "GET"
controller.params = {}
controller.process_action(:index)
raise "probe: #{controller.body}" unless controller.body == "HEY QUIET Hey 7 sent hour sent hour relayed hour relayed hour relayed hour 7 sent hour"
puts "literal send passed"
"##,
    );
    run.assert_passes();
    assert!(run.stdout.contains("literal send passed"), "{}", run.stdout);
}

/// A String the app reopens with a `send` of its own keeps `send` on a String receiver (only the emit is checked:
/// running it would hand the whole app that `send`).
#[test]
fn a_reopened_builtin_s_own_send_keeps_send() {
    let (tree, errors) = app()
        .write("lib/string_send.rb", "class String\n  def send(name, *args)\n    \"own #{name}\"\n  end\nend\n")
        .emit(roundhouse::project::BuildTarget::Ruby);
    assert!(errors.is_empty(), "{errors:?}");
    let probe = std::fs::read_to_string(tree.join("app/models/probe.rb")).expect("emitted probe");
    assert!(probe.contains(r#""quiet".send(:upcase)"#), "{probe}");
}

/// Either Boolean class: `true.send(:to_s)` reaches TrueClass's own `send` when the app defines one.
#[test]
fn a_reopened_boolean_s_own_send_keeps_send() {
    let (tree, errors) = app()
        .write("lib/true_send.rb", "class TrueClass\n  def send(name, *args)\n    \"own\"\n  end\nend\n")
        .edit("lib/probe.rb", "  def typed\n", "  def flagged\n    true.send(:to_s)\n  end\n\n  def typed\n")
        .emit(roundhouse::project::BuildTarget::Ruby);
    assert!(errors.is_empty(), "{errors:?}");
    let probe = std::fs::read_to_string(tree.join("app/models/probe.rb")).expect("emitted probe");
    assert!(probe.contains("true.send(:to_s)"), "{probe}");
}
