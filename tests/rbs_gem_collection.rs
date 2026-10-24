//! gem_rbs_collection signatures for a source gem land on
//! `App.rbs_signatures` as the lowest layer: the app's own `sig/`
//! overrides them, and a method the collection answers `untyped` is
//! left to inference.

use std::fs;
use std::path::Path;

use roundhouse::ident::{ClassId, Symbol};
use roundhouse::ingest::ingest_app;
use roundhouse::ty::Ty;

fn write(path: &Path, content: &str) {
    fs::create_dir_all(path.parent().unwrap()).expect("create parent dir");
    fs::write(path, content).expect("write file");
}

fn ret(ty: &Ty) -> &Ty {
    let Ty::Fn { ret, .. } = ty else { panic!("expected Ty::Fn, got {ty:?}") };
    ret
}

#[test]
fn collection_signatures_are_the_base_layer_for_source_gems() {
    let dir = std::env::temp_dir().join("roundhouse-rbs-gem-collection");
    let _ = fs::remove_dir_all(&dir);
    write(&dir.join("db/schema.rb"), "ActiveRecord::Schema.define(version: 1) do\nend\n");
    write(
        &dir.join("Gemfile.lock"),
        "GEM\n  remote: https://rubygems.org/\n  specs:\n    greeter (0.1.0)\n\nPLATFORMS\n  ruby\n\nDEPENDENCIES\n  greeter\n",
    );
    write(&dir.join(".bundle/config"), "---\nBUNDLE_PATH: \"vendor/bundle\"\n");
    write(&dir.join("roundhouse.yml"), "extensions:\n  - config/roundhouse\n");
    write(&dir.join("config/roundhouse/gems.yml"), "source_gems:\n  - {gem: \"greeter\"}\n");
    write(
        &dir.join("vendor/bundle/ruby/3.4.0/gems/greeter-0.1.0/lib/greeter.rb"),
        "class Greeter\n  def hello = \"hi\"\n  def count = 1\n  def anything = nil\nend\n",
    );
    write(&dir.join("rbs_collection.yaml"), "path: vendor/rbs\n");
    let collection = dir.join("vendor/rbs/greeter");
    write(
        &collection.join("0.1/greeter.rbs"),
        "class Greeter\n  def hello: () -> String\n  def count: () -> String\n  def anything: () -> untyped\nend\n",
    );
    write(&collection.join("0.2/greeter.rbs"), "class Greeter\n  def hello: () -> Integer\nend\n");
    write(&dir.join("sig/greeter.rbs"), "class Greeter\n  def count: () -> Integer\nend\n");

    let app = ingest_app(&dir).expect("ingest");
    let greeter = ClassId(Symbol::from("Greeter"));
    let methods = app.rbs_signatures.get(&greeter).unwrap_or_else(|| panic!("no Greeter: {:?}", app.rbs_signatures.keys().collect::<Vec<_>>()));
    let methods: std::collections::HashMap<_, _> = methods.iter().map(|(name, ty)| (name.as_str().to_string(), ty)).collect();
    assert_eq!(ret(methods.get("hello").expect("hello from the 0.1 collection")), &Ty::Str, "{methods:?}");
    assert_eq!(ret(methods.get("count").expect("count from sig/")), &Ty::Int, "{methods:?}");
    assert!(!methods.contains_key("anything"), "an untyped answer is left to inference: {methods:?}");
}
