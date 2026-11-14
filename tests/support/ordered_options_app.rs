//! `ActiveSupport::OrderedOptions` in app code: a name no method answers reads or writes its key, as Rails'
//! `method_missing` does, on the ruby family's Hash and on Spinel's Hash-holding class alike.

use super::emit_and_run;

pub const SCRIPT: &str = "p SettingsBox.report\n";

pub const EXPECTED: &str = "[\"app/components\", false, \"blue\", \"blue\", nil, 3, \"app/components\"]\n";

pub fn app() -> emit_and_run::Overlay {
    emit_and_run::real_blog().write(
        "app/lib/settings_box.rb",
        "class SettingsBox\n  def self.options\n    options = ActiveSupport::OrderedOptions.new\n    options.merge!({ path: \"app/components\", enabled: false })\n    options.color = \"blue\"\n    options\n  end\n\n  def self.report\n    o = options\n    [o.path, o.enabled, o.color, o[:color], o.missing, o.keys.size, o.path!]\n  end\nend\n",
    )
}
