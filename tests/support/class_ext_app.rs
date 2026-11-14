//! ActiveSupport's `Class#descendants` and `module_parents` on a class, as Rails 8.1 answers them, on the ruby family
//! and on Spinel (whose runtime reopens `Class`: it answers no reopened `Module` method on a class).

use super::emit_and_run;

pub const SCRIPT: &str = "p Critter.descendants.map(&:name).sort\np Zoo::Keeper::Cat.module_parents.map(&:name)\np Zoo::Keeper::Cat.module_parent.name\n";

pub const EXPECTED: &str = "[\"Dog\", \"Puppy\", \"Zoo::Keeper::Cat\"]\n[\"Zoo::Keeper\", \"Zoo\", \"Object\"]\n\"Zoo::Keeper\"\n";

pub fn app() -> emit_and_run::Overlay {
    emit_and_run::real_blog()
        .write("app/lib/critter.rb", "class Critter\n  def sound = \"...\"\nend\n")
        .write("app/lib/dog.rb", "class Dog < Critter\nend\n")
        .write("app/lib/puppy.rb", "class Puppy < Dog\nend\n")
        .write("app/lib/zoo/keeper/cat.rb", "module Zoo\n  module Keeper\n    class Cat < Critter\n    end\n  end\nend\n")
}
