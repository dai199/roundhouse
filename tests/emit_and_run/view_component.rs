//! view_component compiled from its installed source: a view's `render(GreetingComponent.new(...))` reaches the gem's
//! `render_in`, and the component's sidecar template, compiled into its `call` when the app is read, writes the markup
//! Rails 8.1.4 renders for it (`render?`, a component method, an ivar and a view helper).

#[path = "../support/prerequisite.rs"]
mod prerequisite;
use super::emit_and_run;

const GEMS: [(&str, &str); 2] = [("view_component", "3.23.2"), ("method_source", "1.1.0")];

fn installed(name: &str, version: &str) -> Option<std::path::PathBuf> {
    let out = std::process::Command::new("ruby")
        .args(["-e", &format!("print Gem::Specification.find_by_name('{name}', '{version}').gem_dir")])
        .output();
    match out {
        Ok(out) if out.status.success() => Some(std::path::PathBuf::from(String::from_utf8_lossy(&out.stdout).into_owned())),
        _ => {
            prerequisite::missing(&format!("ruby with {name} {version} installed"));
            None
        }
    }
}

fn files(root: &std::path::Path, dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
    for entry in std::fs::read_dir(dir).into_iter().flatten().flatten() {
        let path = entry.path();
        if path.is_dir() {
            files(root, &path, out);
        } else {
            out.push(path.strip_prefix(root).unwrap().to_path_buf());
        }
    }
}

const COMPONENT: &str = "class GreetingComponent < ViewComponent::Base\n  def initialize(name:, count: 0)\n    @name = name\n    @count = count\n  end\n\n  def shout = @name.upcase\n\n  def render? = @count >= 0\nend\n";

const TEMPLATE: &str = "<p class=\"greeting\">Hello, <%= shout %>! (<%= pluralize(@count, \"article\") %>)</p>\n";

const SCRIPT: &str = r#"io = +""
Views::Articles.index_into(io, Article.all, nil, nil)
expected = "<p class=\"greeting\">Hello, BLOG! (0 articles)</p>"
raise "component:\n#{io}" unless io.include?(expected)
puts "view_component passed"
"#;

fn app() -> Option<emit_and_run::Overlay> {
    let fixture = roundhouse::fixtures::real_blog();
    let lock = std::fs::read_to_string(fixture.join("Gemfile.lock")).unwrap();
    let lock = lock
        // Not a bare "    rack (": a dependency line ("      rack (>= 2.2.4)") holds it too.
        .replacen(
            "\n    rack (",
            "\n    method_source (1.1.0)\n    view_component (3.23.2)\n      activesupport (>= 5.2.0, < 8.1)\n      concurrent-ruby (~> 1)\n      method_source (~> 1.0)\n    rack (",
            1,
        )
        .replacen("DEPENDENCIES\n", "DEPENDENCIES\n  view_component\n", 1);
    let gemfile = std::fs::read_to_string(fixture.join("Gemfile")).unwrap() + "gem \"view_component\"\n";
    let mut overlay = emit_and_run::real_blog()
        .write("Gemfile", &gemfile)
        .write("Gemfile.lock", &lock)
        .write(".bundle/config", "---\nBUNDLE_PATH: \"vendor/bundle\"\n")
        .write("app/components/greeting_component.rb", COMPONENT)
        .write("app/components/greeting_component.html.erb", TEMPLATE)
        .edit(
            "app/views/articles/index.html.erb",
            "<% content_for :title, \"Articles\" %>\n",
            "<% content_for :title, \"Articles\" %>\n<%= render(GreetingComponent.new(name: \"blog\", count: @articles.size)) %>\n",
        );
    for (name, version) in GEMS {
        let gem = installed(name, version)?;
        let mut list = Vec::new();
        files(&gem, &gem.join("lib"), &mut list);
        files(&gem, &gem.join("app"), &mut list);
        for file in list {
            let Ok(text) = std::fs::read_to_string(gem.join(&file)) else { continue };
            overlay = overlay.write(&format!("vendor/bundle/ruby/3.4.0/gems/{name}-{version}/{}", file.display()), &text);
        }
    }
    Some(overlay)
}

/// What the gem still does that no tree answers, none of which this render reaches: its previews (a development-only
/// page of the engine), its test helpers' conditional visibility, the engine's Railtie and `Rails::Engine` itself.
const KNOWN_GAPS: [&str; 9] = [
    "@render_args has no known type",
    "@example_name has no known type",
    "@preview has no known type",
    "@view_renderer has no known type",
    "render with a block not supported (all targets)",
    "conditional or dynamic visibility declarations are not modeled",
    "railtie middleware not supported (all targets)",
    "constant not supported (all targets): Rails::Engine",
    "render of an object not supported (all targets)",
];

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
