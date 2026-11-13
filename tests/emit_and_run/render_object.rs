//! A view's `render` of an object: Rails asks it to render itself (`render_in(view_context)`), which no view lowering
//! models yet. It is reported on every target, not emitted as a bare `render(…)` the view's module does not answer.

use roundhouse::build_target::BuildTarget;

use super::emit_and_run;

const BADGE: &str = "class Badge\n  def initialize(label)\n    @label = label\n  end\n\n  def render_in(_view_context)\n    \"<b>#{@label}</b>\"\n  end\nend\n";

fn app() -> emit_and_run::Overlay {
    emit_and_run::real_blog().write("app/models/badge.rb", BADGE).edit(
        "app/views/articles/index.html.erb",
        "<% content_for :title, \"Articles\" %>\n",
        "<% content_for :title, \"Articles\" %>\n<%= render(Badge.new(\"new\")) %>\n",
    )
}

#[test]
fn rendering_an_object_is_reported_on_every_target() {
    for target in [BuildTarget::Ruby, BuildTarget::Spinel] {
        let (_, errors) = app().emit(target);
        assert!(errors.iter().any(|e| e.contains("render of an object not supported")), "{target:?}: {errors:#?}");
    }
}
