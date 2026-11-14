# frozen_string_literal: true

# `ActiveSupport::OrderedOptions` for the Spinel tree, after activesupport 8.1
# `lib/active_support/ordered_options.rb` (MIT license, Copyright (c) David
# Heinemeier Hansson). Rails' is a Hash whose `method_missing` reads and
# writes keys; Spinel subclasses no builtin and dispatches no
# `method_missing`, so this one holds its Hash, and a call no method answers
# is written as the `method_missing` call Ruby makes
# (`lower::method_missing_calls`). The ruby family ships Rails' own shape
# (`ruby_overlay/runtime/active_support_ordered_options.rb`).
module ActiveSupport
  class OrderedOptions
    def initialize(default = nil)
      @values = {}
      @default = default
    end

    def [](key)
      @values.fetch(key.to_sym, @default)
    end

    def []=(key, value)
      @values[key.to_sym] = value
    end

    # Hash#merge!: the keys as given, as Rails' does (only `[]=` symbolizes).
    def merge!(other)
      other.each { |key, value| @values[key] = value }
      self
    end

    def update(other)
      merge!(other)
    end

    def keys
      @values.keys
    end

    def key?(key)
      @values.key?(key)
    end

    def fetch(key)
      @values.fetch(key)
    end

    def to_h
      @values.dup
    end

    def empty?
      @values.empty?
    end

    def method_missing(name, *args)
      text = name.to_s
      if text.end_with?("=")
        self[text[0, text.length - 1].to_s] = args[0]
      elsif text.end_with?("!")
        key = text[0, text.length - 1].to_s
        value = self[key]
        raise KeyError, ":#{key} is blank" if value.nil? || value == false || value == "" || value == [] || value == {}
        value
      else
        self[text]
      end
    end

    def respond_to_missing?(_name, _include_private)
      true
    end
  end
end
