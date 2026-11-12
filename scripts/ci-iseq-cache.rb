# Not bootsnap: the unit job's rubies boot emitted trees and oracles, not one Bundler app, and need only its compile cache.
if RUBY_ENGINE == "ruby" && (cache = ENV["RH_ISEQ_CACHE"]) && !cache.empty?
  require "digest/sha2"
  Digest::SHA256.hexdigest("")

  module RoundhouseIseqWarned
    def warn(*args, **kw)
      Thread.current[:roundhouse_iseq_warned] = true
      super
    end
  end
  Warning.singleton_class.prepend(RoundhouseIseqWarned)

  class << RubyVM::InstructionSequence
    def load_iseq(path)
      source = File.binread(path)
      key = File.join(ENV["RH_ISEQ_CACHE"], Digest::SHA256.hexdigest("#{RUBY_DESCRIPTION}\0#{RubyVM::InstructionSequence.compile_option}\0#{$VERBOSE.inspect}\0#{path}\0#{source}"))
      begin
        return RubyVM::InstructionSequence.load_from_binary(File.binread(key))
      rescue Errno::ENOENT
      end
      Thread.current[:roundhouse_iseq_warned] = false
      iseq = RubyVM::InstructionSequence.compile_file(path)
      # Not cached when compiling warned: a cached load would not warn again.
      unless Thread.current[:roundhouse_iseq_warned]
        tmp = "#{key}.#{Process.pid}.#{Thread.current.object_id}"
        File.binwrite(tmp, iseq.to_binary)
        File.rename(tmp, key)
      end
      iseq
    rescue SyntaxError
      nil
    end
  end
end
