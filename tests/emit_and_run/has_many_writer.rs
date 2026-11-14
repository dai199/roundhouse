//! A has_many's writer from app code (`author.posts = records`), as Rails
//! 8.1.4's `replace_records` runs it: on a persisted owner, in one
//! transaction, a record left out is removed per `dependent:` (destroyed,
//! deleted, or its key nulled), each new one is saved with the key, and a
//! record that can't be saved raises `RecordNotSaved` and rolls the
//! replacement back. A new owner holds the list and saves it after itself
//! (`ingest::has_many_hold`). The probe passes unchanged on ActiveRecord
//! 8.1.4.

use super::emit_and_run;

fn app() -> emit_and_run::Overlay {
    emit_and_run::empty_app()
        .write("app/models/application_record.rb", r##"class ApplicationRecord < ActiveRecord::Base
  self.abstract_class = true
end
"##)
        .write("app/models/author.rb", r##"class Author < ApplicationRecord
  has_many :posts
  has_many :drafts, dependent: :destroy
end
"##)
        .write("app/models/draft.rb", r##"class Draft < ApplicationRecord
  belongs_to :author, optional: true
end
"##)
        .write("app/models/post.rb", r##"class Post < ApplicationRecord
  belongs_to :author, optional: true
  validates :title, presence: true
end
"##)
        .write("app/models/probe.rb", r##"module Probe
  def self.check(label, ok)
    raise "#{label} differs from Rails" unless ok
  end

  def self.run
    # A persisted owner: a record left out has its key nulled, a new one
    # is saved with the key, and the reader answers the list.
    a = Author.create!(name: "a")
    old = Post.create!(title: "old", author: a)
    kept = Post.create!(title: "kept", author: a)
    n = Post.new(title: "new")
    a.posts = [kept, n]
    check("new saved", n.persisted? && n.author_id == a.id)
    check("old nulled", Post.find(old.id).author_id.nil?)
    check("kept", Post.find(kept.id).author_id == a.id)
    check("reader", a.posts.map(&:title).sort == ["kept", "new"])
    check("fresh read", Author.find(a.id).posts.map(&:title).sort == ["kept", "new"])
    # dependent: :destroy destroys a record left out.
    d = Draft.create!(title: "d", author: a)
    a.drafts = [Draft.new(title: "e")]
    check("dropped draft destroyed", !Draft.exists?(d.id))
    check("drafts", Draft.where(author_id: a.id).pluck(:title) == ["e"])
    # An empty list removes them all.
    a.posts = []
    check("emptied", Post.where(author_id: a.id).count == 0 && a.posts.size == 0)
    # A new owner holds the list and saves it with its key after itself.
    o = Author.new(name: "o")
    h = Post.new(title: "held")
    o.posts = [h]
    check("held unsaved", !h.persisted?)
    check("held listed", o.posts.size == 1)
    o.save!
    check("held saved", h.persisted? && Post.find(h.id).author_id == o.id)
    # A record that can't be saved raises and rolls the replacement back.
    b = Author.create!(name: "b")
    first = Post.create!(title: "first", author: b)
    raised = false
    begin
      b.posts = [Post.new(title: "")]
    rescue ActiveRecord::RecordNotSaved
      raised = true
    end
    check("record not saved", raised)
    check("rolled back", Post.find(first.id).author_id == b.id)
  end
end
"##)
        .write("db/schema.rb", r##"ActiveRecord::Schema.define do
  create_table "authors", force: :cascade do |t|
    t.string "name"
  end
  create_table "posts", force: :cascade do |t|
    t.string "title"
    t.integer "author_id"
  end
  create_table "drafts", force: :cascade do |t|
    t.string "title"
    t.integer "author_id"
  end
end
"##)
}

const SCRIPT: &str = "Probe.run\nputs \"probe passed\"\n";

#[test]
fn a_has_many_writer_replaces_as_rails_does() {
    let run = app().run_ruby(SCRIPT);
    run.assert_passes();
    assert!(run.stdout.contains("probe passed"), "{}", run.stdout);
}

#[test]
#[ignore = "requires the Spinel toolchain"]
fn a_has_many_writer_replaces_as_rails_does_on_spinel() {
    let run = app().run_spinel_with_db(SCRIPT);
    run.assert_passes();
    assert!(run.stdout.contains("probe passed"), "{}", run.stdout);
}
