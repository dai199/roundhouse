//! When a source gem's file runs relative to another, from the order its `require`s and `autoload`s load them.
//!
//! Ruby's lexical constant lookup sees only what is defined by the time the reference is evaluated. A reference
//! evaluated while a file loads (a class body, a constant's value) therefore names an enclosing namespace's
//! constant only once some file has defined it: Doorkeeper's `lib/doorkeeper.rb` requires `doorkeeper/config` on
//! its third line and opens `module Doorkeeper::Rails` on line 110, so `Rails` in the Config class body is the
//! top-level one. Each gem's files get a time from a walk of its entry file, in source order: a `require` loads
//! its file there, an `autoload` registers a file that loads no earlier than that. App files load as the
//! autoloader asks, in no order this can know, so they are compared within the file only.

use crate::det::HashMap;
use crate::ingest::prism_api::PrismLocation;

#[derive(Default)]
pub(super) struct LoadOrder {
    /// Per gem file, the times of its loading statements by offset, ascending, and its own start.
    files: HashMap<String, FileTimes>,
    /// A file only an `autoload` names: the earliest time it can load.
    autoloaded: HashMap<String, (u32, usize)>,
}

struct FileTimes {
    start: u32,
    events: Vec<(u32, u32)>,
    /// The entry whose walk loaded it: two gems' entries load in Bundler's order, which is not compared.
    entry: usize,
}

struct Load {
    offset: u32,
    end: u32,
    target: String,
    eager: bool,
}

impl LoadOrder {
    pub(super) fn of(sources: &[crate::span::SourceFile]) -> Self {
        let gem: Vec<&crate::span::SourceFile> =
            sources.iter().filter(|s| s.path.ends_with(".rb") && crate::ingest::absence_guards::is_source_gem_path(&s.path)).collect();
        if gem.is_empty() {
            return Self::default();
        }
        let paths: crate::det::HashSet<&str> = gem.iter().map(|s| s.path.as_str()).collect();
        let roots: Vec<&str> = {
            let mut roots: Vec<&str> = gem.iter().filter_map(|s| s.path.find("/lib/").map(|at| &s.path[..at + 5])).collect();
            roots.sort_unstable();
            roots.dedup();
            roots
        };
        let resolve = |from: &str, feature: &str, relative: bool| -> Option<String> {
            let feature = feature.strip_suffix(".rb").unwrap_or(feature);
            if relative {
                let dir = from.rsplit_once('/').map_or("", |(dir, _)| dir);
                let joined = normalize(&format!("{dir}/{feature}.rb"));
                return paths.contains(joined.as_str()).then_some(joined);
            }
            roots.iter().map(|root| format!("{root}{feature}.rb")).find(|candidate| paths.contains(candidate.as_str()))
        };
        let loads: HashMap<&str, Vec<Load>> = gem.iter().map(|s| (s.path.as_str(), loads_of(&s.path, &s.text, &resolve))).collect();
        let targets: crate::det::HashSet<&str> = loads.values().flatten().map(|l| l.target.as_str()).collect();
        let mut entries: Vec<&str> = gem.iter().map(|s| s.path.as_str()).filter(|p| !targets.contains(p)).collect();
        entries.sort_unstable();
        let mut order = Self::default();
        let mut clock = 0u32;
        for (entry, path) in entries.into_iter().enumerate() {
            order.walk(path, entry, &loads, &mut clock);
        }
        // What an autoloaded file requires loads with it, no earlier than its registration either.
        let mut pending: Vec<(String, u32, usize)> = order.autoloaded.iter().map(|(path, (t, entry))| (path.clone(), *t, *entry)).collect();
        pending.sort();
        while let Some((path, at, entry)) = pending.pop() {
            for load in loads.get(path.as_str()).into_iter().flatten() {
                if !order.files.contains_key(&load.target) && !order.autoloaded.contains_key(&load.target) {
                    order.autoloaded.insert(load.target.clone(), (at, entry));
                    pending.push((load.target.clone(), at, entry));
                }
            }
        }
        order
    }

    fn walk(&mut self, path: &str, entry: usize, loads: &HashMap<&str, Vec<Load>>, clock: &mut u32) {
        if self.files.contains_key(path) {
            return;
        }
        *clock += 1;
        self.files.insert(path.to_string(), FileTimes { start: *clock, events: Vec::new(), entry });
        for load in loads.get(path).into_iter().flatten() {
            *clock += 1;
            if let Some(times) = self.files.get_mut(path) {
                times.events.push((load.offset, *clock));
            }
            if load.eager {
                self.walk(&load.target, entry, loads, clock);
                // What follows the `require` runs after everything it loaded.
                *clock += 1;
                if let Some(times) = self.files.get_mut(path) {
                    times.events.push((load.end, *clock));
                }
            } else if !self.files.contains_key(&load.target) {
                self.autoloaded.entry(load.target.clone()).or_insert((*clock, entry));
            }
        }
    }

    /// When the statement at `offset` in `path` runs, and the walk that loads it.
    fn time(&self, path: &str, offset: u32) -> Option<(u32, usize)> {
        if let Some(times) = self.files.get(path) {
            let at = times.events.iter().take_while(|(event, _)| *event <= offset).last().map_or(times.start, |(_, t)| *t);
            return Some((at, times.entry));
        }
        self.autoloaded.get(path).copied()
    }

    /// Whether a definition at `def_offset` in `def_path` runs only after the reference at `at` in `path` is
    /// evaluated. Same file: after it in the source. Two files of one gem: later in its load walk.
    pub(super) fn later(&self, def_path: &str, def_offset: u32, path: &str, at: u32) -> bool {
        if def_path == path {
            return def_offset > at;
        }
        match (self.time(def_path, def_offset), self.files.get(path).and(self.time(path, at))) {
            (Some((defined, a)), Some((read, b))) => a == b && defined > read,
            _ => false,
        }
    }
}

fn normalize(path: &str) -> String {
    let mut out: Vec<&str> = Vec::new();
    for part in path.split('/') {
        match part {
            ".." => {
                out.pop();
            }
            "." => {}
            _ => out.push(part),
        }
    }
    out.join("/")
}

/// The `require`/`require_relative`/`autoload` calls a file runs as it loads, of a literal feature this tree holds.
fn loads_of(path: &str, text: &str, resolve: &dyn Fn(&str, &str, bool) -> Option<String>) -> Vec<Load> {
    struct Visit<'a> {
        path: &'a str,
        resolve: &'a dyn Fn(&str, &str, bool) -> Option<String>,
        out: Vec<Load>,
    }
    impl<'pr> ruby_prism::Visit<'pr> for Visit<'_> {
        // Not a method's body: it runs when called, not as the file loads.
        fn visit_def_node(&mut self, _node: &ruby_prism::DefNode<'pr>) {}

        fn visit_call_node(&mut self, node: &ruby_prism::CallNode<'pr>) {
            let name = node.name();
            let name = name.as_slice();
            if node.receiver().is_none_or(|r| r.as_self_node().is_some()) && matches!(name, b"require" | b"require_relative" | b"autoload") {
                let args: Vec<_> = node.arguments().map(|a| a.arguments().iter().collect()).unwrap_or_default();
                let feature = match (name, args.as_slice()) {
                    (b"autoload", [_, feature]) => feature.as_string_node(),
                    (b"require" | b"require_relative", [feature]) => feature.as_string_node(),
                    _ => None,
                };
                if let Some(feature) = feature {
                    let feature = String::from_utf8_lossy(feature.unescaped()).into_owned();
                    if let Some(target) = (self.resolve)(self.path, &feature, name == b"require_relative") {
                        let location = node.location();
                        self.out.push(Load { offset: location.start_offset() as u32, end: location.end_offset() as u32, target, eager: name != b"autoload" });
                    }
                }
            }
            ruby_prism::visit_call_node(self, node);
        }
    }
    let parsed = ruby_prism::parse(text.as_bytes());
    let mut visit = Visit { path, resolve, out: Vec::new() };
    ruby_prism::Visit::visit(&mut visit, &parsed.node());
    visit.out.sort_by_key(|load| load.offset);
    visit.out
}

#[cfg(test)]
mod tests {
    use super::LoadOrder;
    use crate::span::SourceFile;

    fn source(path: &str, text: &str) -> SourceFile {
        SourceFile { path: format!("/app/.roundhouse/source_gems/probe-1.0/lib/{path}"), text: text.to_string() }
    }

    fn path(p: &str) -> String {
        format!("/app/.roundhouse/source_gems/probe-1.0/lib/{p}")
    }

    /// Doorkeeper's shape: the entry requires its config first, then opens a namespace and autoloads files that
    /// require more of it.
    #[test]
    fn a_file_required_first_runs_before_what_the_entry_defines_and_autoloads_after() {
        let order = LoadOrder::of(&[
            source("probe.rb", "require \"probe/config\"\nmodule Probe\n  module Rails\n    autoload :Routes, \"probe/rails/routes\"\n  end\nend\n"),
            source("probe/config.rb", "module Probe\n  class Config\n    FORCE = Rails.env\n  end\nend\n"),
            source("probe/rails/routes.rb", "require \"probe/rails/routes/mapper\"\nmodule Probe\n  module Rails\n    class Routes\n    end\n  end\nend\n"),
            source("probe/rails/routes/mapper.rb", "module Probe\n  module Rails\n    class Mapper\n    end\n  end\nend\n"),
        ]);
        let read = 40;
        assert!(order.later(&path("probe.rb"), 30, &path("probe/config.rb"), read));
        assert!(order.later(&path("probe/rails/routes.rb"), 50, &path("probe/config.rb"), read));
        assert!(order.later(&path("probe/rails/routes/mapper.rb"), 10, &path("probe/config.rb"), read));
        // The entry's own statements before the require ran first.
        assert!(!order.later(&path("probe.rb"), 0, &path("probe/config.rb"), read));
        // Within one file, by position.
        assert!(order.later(&path("probe/config.rb"), 60, &path("probe/config.rb"), read));
        assert!(!order.later(&path("probe/config.rb"), 10, &path("probe/config.rb"), read));
    }

    /// An app file loads as the autoloader asks: only its own later lines are known to run after it.
    #[test]
    fn an_app_file_is_ordered_within_itself_only() {
        let order = LoadOrder::of(&[SourceFile { path: "/app/app/models/a.rb".into(), text: "class A\nend\n".into() }]);
        assert!(!order.later("/app/app/models/b.rb", 100, "/app/app/models/a.rb", 0));
        assert!(order.later("/app/app/models/a.rb", 100, "/app/app/models/a.rb", 0));
    }
}
