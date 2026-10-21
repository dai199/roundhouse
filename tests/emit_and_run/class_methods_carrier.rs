use super::emit_and_run;

const DISABLER: &str = r#"module Widgets
  class Tag
    def render
      "tag"
    end

    module Disabler
      module ClassMethods
        attr_reader :disabled_tags
      end

      def self.prepended(base)
        base.extend(ClassMethods)
      end

      def render
        "disabled #{self.class.disabled_tags.inspect}, else #{super}"
      end
    end
  end

  class Render < Tag
    prepend Tag::Disabler
    @disabled_tags = ["include"]
  end
end
"#;

#[test]
fn a_class_methods_module_a_prepended_hook_extends_stays_a_module() {
    emit_and_run::real_blog()
        .write("app/models/widgets.rb", DISABLER)
        .run_ruby(
            r#"
raise "carrier lost: #{Widgets::Render.disabled_tags.inspect}" unless Widgets::Render.disabled_tags == ["include"]
got = Widgets::Render.new.render
raise "render: #{got}" unless got == 'disabled ["include"], else tag'
raise "the module is its own" unless Widgets::Tag::Disabler::ClassMethods.is_a?(Module)
puts "ok"
"#,
        )
        .assert_passes();
}
