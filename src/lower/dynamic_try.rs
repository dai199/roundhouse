//! `recv.try(name, ...)` whose name is a value (Devise's `apply_to_attribute_or_variable`: `send(attr).try(method)`),
//! written as ActiveSupport's own body at the call: nil for a nil receiver, else `public_send(name, ...)` where the
//! receiver `respond_to?(name)`, else nil. A literal name is `try_guard`'s, and a name a loop binds `send_dispatch`'s.
//! The Ruby family's trees only: Spinel resolves the `respond_to?` and `public_send` of a computed name over the
//! names the program spells.

use crate::analyze::ClassInfo;
use crate::app::App;
use crate::det::HashMap;
use crate::expr::{Expr, ExprNode, LValue, Literal};
use crate::ident::{ClassId, Symbol, VarId};
use crate::ty::Ty;

pub fn apply_dynamic_try(app: &mut App, registry: &HashMap<ClassId, ClassInfo>, target: crate::build_target::BuildTarget) {
    if !target.has_runtime_relation() {
        return;
    }
    crate::lower::for_each_emit_body(app, &mut |e| rewrite(e, registry));
}

fn rewrite(e: &mut Expr, registry: &HashMap<ClassId, ClassInfo>) {
    e.node.for_each_child_mut(&mut |child| rewrite(child, registry));
    let ExprNode::Send { recv: Some(recv), method, args, block: None, .. } = &*e.node else { return };
    if method.as_str() != "try" {
        return;
    }
    // Not ActiveSupport's on a receiver whose class has a `try` of its own (parslet's atoms' `try(source, context,
    // consume_all)`): that `try` overrides Object's, so the call is that method's.
    let own = own_try(registry, recv.ty.as_ref());
    if own == Own::Always {
        return;
    }
    let Some((name, rest)) = args.split_first() else { return };
    // Not a name read twice that could run code: it is asked once and sent once.
    if !matches!(&*name.node, ExprNode::Var { .. } | ExprNode::Ivar { .. }) {
        return;
    }
    let span = e.span;
    let held = Symbol::from(format!("__rh_try_{}", span.start));
    let read = || Expr::new(span, ExprNode::Var { id: VarId(0), name: held.clone() });
    let nil = || Expr::new(span, ExprNode::Lit { value: Literal::Nil });
    let send = |method: &str, args: Vec<Expr>| {
        Expr::new(span, ExprNode::Send { recv: Some(read()), method: Symbol::from(method), args, block: None, parenthesized: true })
    };
    // Not the other arguments evaluated in the send's branch only: Ruby evaluates them, in order, before `try` runs.
    let mut binds = Vec::new();
    let mut sent = vec![name.clone()];
    for (i, arg) in rest.iter().enumerate() {
        // Not a variable either: a later argument may assign it before the send reads it.
        if matches!(&*arg.node, ExprNode::Lit { .. } | ExprNode::SelfRef) {
            sent.push(arg.clone());
            continue;
        }
        let local = Symbol::from(format!("__rh_try_{}_{i}", span.start));
        binds.push(Expr::new(span, ExprNode::Assign { target: LValue::Var { id: VarId(0), name: local.clone() }, value: arg.clone() }));
        sent.push(Expr::new(span, ExprNode::Var { id: VarId(0), name: local }));
    }
    let sent_again = sent.clone();
    let answer = Expr::new(span, ExprNode::If {
        cond: send("respond_to?", vec![name.clone()]),
        then_branch: send("public_send", sent),
        else_branch: nil(),
    });
    let mut guarded = Expr::new(span, ExprNode::If { cond: send("nil?", Vec::new()), then_branch: nil(), else_branch: answer });
    // Not ActiveSupport's for a receiver that may be of such a class: the emitted runtime defines no Object#try, so
    // only a class's own `try` answers `respond_to?(:try)`.
    if own == Own::Maybe {
        let try_sym = Expr::new(span, ExprNode::Lit { value: Literal::Sym { value: Symbol::from("try") } });
        guarded = Expr::new(span, ExprNode::If {
            cond: send("respond_to?", vec![try_sym]),
            then_branch: send("try", sent_again),
            else_branch: guarded,
        });
    }
    let hold = Expr::new(span, ExprNode::Assign { target: LValue::Var { id: VarId(0), name: held.clone() }, value: recv.clone() });
    let mut exprs = vec![hold];
    exprs.extend(binds);
    exprs.push(guarded);
    *e = Expr::new(span, ExprNode::Seq { exprs });
}

#[derive(PartialEq)]
enum Own {
    Never,
    Maybe,
    Always,
}

fn own_try(registry: &HashMap<ClassId, ClassInfo>, ty: Option<&Ty>) -> Own {
    let any_class = || {
        registry.iter().any(|(id, cls)| {
            !matches!(id.0.as_str(), "Object" | "Kernel" | "BasicObject" | "NilClass" | "Module" | "Class")
                && cls.instance_methods.contains_key(&Symbol::from("try"))
        })
    };
    match ty {
        Some(Ty::Class { id, .. }) => {
            if crate::lower::send_dispatch::defines_own(registry, id, "try", false)
                || crate::lower::send_dispatch::defines_own(registry, id, "try", true)
            {
                Own::Always
            } else {
                Own::Never
            }
        }
        Some(Ty::Union { variants }) => {
            let each: Vec<Own> = variants.iter().map(|v| own_try(registry, Some(v))).collect();
            if each.iter().all(|o| *o == Own::Always) {
                Own::Always
            } else if each.iter().all(|o| *o == Own::Never) {
                Own::Never
            } else {
                Own::Maybe
            }
        }
        // Not left to ActiveSupport when the class is not known: any class with its own `try` may be the receiver.
        None | Some(Ty::Var { .. } | Ty::Untyped | Ty::SelfInstance) if any_class() => Own::Maybe,
        _ => Own::Never,
    }
}
