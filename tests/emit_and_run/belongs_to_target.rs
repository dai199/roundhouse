//! A belongs_to's loaded target goes stale when its key is written
//! through `self[:fk] =` or `write_attribute` to another value, and a
//! preload keeps a target already loaded (Rails' preloader skips a loaded
//! association). The probe's expected values are ActiveRecord 8.1.4's
//! answers.

use super::emit_and_run;

fn app() -> emit_and_run::Overlay {
    emit_and_run::empty_app()
        .write("app/models/application_record.rb", r##"class ApplicationRecord < ActiveRecord::Base
  self.abstract_class = true
end
"##)
        .write("app/models/author.rb", r##"class Author < ApplicationRecord
  has_many :posts
end
"##)
        .write("app/models/post.rb", r##"class Post < ApplicationRecord
  belongs_to :author, optional: true
end
"##)
        .write("app/models/probe.rb", r##"module Probe
  def self.check(label, ok)
    raise "#{label} differs from Rails" unless ok
  end

  def self.reset
    [Post, Author].each(&:delete_all)
  end

  # Rails: ["b", "a"]
  def self.index_write_makes_belongs_to_stale
    a = Author.create!(name: "a"); b = Author.create!(name: "b"); p = Post.create!(title: "t", author: a); p.author
    p[:author_id] = b.id; n1 = p.author.name; p.write_attribute(:author_id, a.id); n2 = p.author.name
    [n1, n2]
  end

  # Rails: [true]
  def self.belongs_to_preload_reuses_loaded
    a = Author.create!(name: "a"); p = Post.create!(title: "t", author: a); p2 = Post.find(p.id); held = p2.author
    ActiveRecord::Associations::Preloader.new(records: [p2], associations: :author).call
    [p2.author.equal?(held)]
  end

  def self.run
    reset
    check("index_write_makes_belongs_to_stale", index_write_makes_belongs_to_stale == ["b", "a"])
    reset
    check("belongs_to_preload_reuses_loaded", belongs_to_preload_reuses_loaded == [true])
  end
end
"##)
        .write("db/schema.rb", r##"ActiveRecord::Schema.define do
  create_table "authors", force: :cascade do |t|
    t.string "name"
    t.boolean "active", default: true
  end
  create_table "posts", force: :cascade do |t|
    t.string "title"
    t.integer "author_id"
    t.boolean "published", default: false
    t.integer "words", default: 0
  end
end
"##)
}

const SCRIPT: &str = "Probe.run\nputs \"probe passed\"\n";

#[test]
fn a_belongs_to_target_stays_or_goes_stale_as_rails_does() {
    let run = app().run_ruby(SCRIPT);
    run.assert_passes();
    assert!(run.stdout.contains("probe passed"), "{}", run.stdout);
}

#[test]
#[ignore = "requires the Spinel toolchain"]
fn a_belongs_to_target_stays_or_goes_stale_as_rails_does_on_spinel() {
    let run = app().run_spinel_with_db(SCRIPT);
    run.assert_passes();
    assert!(run.stdout.contains("probe passed"), "{}", run.stdout);
}
