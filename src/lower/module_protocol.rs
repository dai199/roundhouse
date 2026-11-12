//! A Module query on a value the analysis cannot prove is a class (simple_form's `at.const_get(mapping)`, whose `at`
//! is `Object` or a constantized namespace) is refused on every target that needs the proof to answer it. CRuby and
//! JRuby run the call as Ruby does, so on those trees the site is emitted as written rather than as the refusal's
//! `raise`, and `bin/roundhouse` drops its diagnostic there.

use crate::App;
use crate::build_target::BuildTarget;
use crate::diagnostic::DiagnosticKind;
use crate::expr::Expr;

pub fn is_unproven_receiver_refusal(kind: &DiagnosticKind) -> bool {
    matches!(kind, DiagnosticKind::Unsupported { construct, detail, .. }
        if construct.as_str() == "Module protocol" && detail.ends_with("requires a proven class/module object receiver"))
}

pub fn lower_for_target(app: &App, target: BuildTarget) -> Option<App> {
    if !matches!(target, BuildTarget::Ruby | BuildTarget::Jruby) {
        return None;
    }
    fn refused(e: &Expr) -> bool {
        let mut found = e.diagnostic().as_ref().is_some_and(is_unproven_receiver_refusal);
        e.node.for_each_child(&mut |c| found |= refused(c));
        found
    }
    let mut found = false;
    crate::lower::for_each_emit_body_ref(app, &mut |body| found |= refused(body));
    if !found {
        return None;
    }
    fn clear(e: &mut Expr) {
        e.node.for_each_child_mut(&mut clear);
        if e.diagnostic().as_ref().is_some_and(is_unproven_receiver_refusal) {
            crate::expr::ExprCold::set_diagnostic(&mut e.cold, None);
        }
    }
    let mut app = app.clone();
    crate::lower::for_each_emit_body(&mut app, &mut clear);
    Some(app)
}
