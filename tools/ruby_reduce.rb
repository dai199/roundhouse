#!/usr/bin/env ruby
# frozen_string_literal: true

# ruby_reduce.rb — shrink a multi-file Ruby reproduction while keeping it
# valid Ruby.
#
# Usage:
#   ruby_reduce.rb --dir DIR --test 'COMMAND' [--files LIST]
#
#   --dir    the directory the reproduction lives in (files are edited in place;
#            copy it first)
#   --test   a shell command run in DIR; exit status 0 means "still reproduces"
#   --files  a file listing the .rb files to shrink, one per line, relative to
#            DIR (default: every .rb under DIR)
#
# Passes, repeated until none makes progress:
#   1. file:      empty a whole file, keeping only its require lines
#   2. statement: delete runs of sibling statements (ddmin over every statement
#                 list: class/module bodies, method bodies, blocks, branches),
#                 outermost lists first
#   3. hoist:     replace an if/unless/while/until/begin with the statements of
#                 one of its branches
#
# Every candidate is re-parsed with Prism and skipped if it has a syntax error,
# so the result stays a program a reader can follow. Candidates already tested
# are skipped by content hash.

require "prism"
require "optparse"
require "digest"
require "open3"

opts = { dir: nil, test: nil, files: nil }
OptionParser.new do |o|
  o.on("--dir DIR") { |v| opts[:dir] = v }
  o.on("--test CMD") { |v| opts[:test] = v }
  o.on("--files LIST") { |v| opts[:files] = v }
end.parse!
abort "--dir and --test are required" unless opts[:dir] && opts[:test]

Dir.chdir(opts[:dir])
FILES = if opts[:files]
          File.read(opts[:files]).split("\n").map(&:strip).reject(&:empty?)
        else
          Dir.glob("**/*.rb").sort
        end
TEST = opts[:test]

# Prism's offsets count bytes; a String indexes characters. Every read is
# binary so a multibyte comment cannot shift a deletion into the code.
def read(f) = File.binread(f)
def write(f, s) = File.binwrite(f, s)
SEEN = {}
$tests = 0

def interesting?
  key = Digest::SHA256.hexdigest(FILES.map { |f| File.exist?(f) ? read(f) : "" }.join("\0"))
  return SEEN[key] if SEEN.key?(key)
  $tests += 1
  _out, status = Open3.capture2e(TEST)
  SEEN[key] = status.success?
end

def valid?(src)
  Prism.parse(src).errors.empty?
end

# Try `candidate` for `file`; keep it when it is valid Ruby and still reproduces.
def try(file, candidate)
  orig = read(file)
  return false if candidate == orig || !valid?(candidate)
  write(file, candidate)
  return true if interesting?
  write(file, orig)
  false
end

def requires_only(src)
  src.lines.select { |l| l =~ /^\s*require(_relative)?\b/ }.join
end

# Every statement list in the file, outermost first, as arrays of
# [start, end] byte ranges (each statement extended to its whole lines so
# deleting it leaves no stray indentation).
def statement_groups(src)
  groups = []
  queue = [[Prism.parse(src).value, 0]]
  until queue.empty?
    node, depth = queue.shift
    next unless node
    if node.is_a?(Prism::StatementsNode)
      ranges = node.body.map { |s| line_range(src, s.location) }
      groups << [depth, ranges] unless ranges.empty?
    end
    node.compact_child_nodes.each { |c| queue << [c, depth + 1] }
  end
  groups.sort_by { |d, r| [d, -r.sum { |a, b| b - a }] }.map(&:last)
end

def line_range(src, loc)
  s = loc.start_offset
  e = loc.end_offset
  s -= 1 while s > 0 && src[s - 1] != "\n" && src[s - 1] =~ /[ \t]/
  e += 1 while e < src.bytesize && src[e] != "\n"
  e += 1 if e < src.bytesize
  [s, e]
end

def delete_ranges(src, ranges)
  out = src.dup
  ranges.sort_by { |s, _| -s }.each { |s, e| out[s...e] = "" }
  out
end

def file_pass
  progress = false
  FILES.each do |f|
    next unless File.exist?(f)
    src = read(f)
    stripped = requires_only(src)
    next if stripped == src
    if try(f, stripped)
      progress = true
      warn "emptied #{f} (#{$tests} tests)"
    end
  end
  progress
end

# ddmin over one statement list; returns true after the first accepted removal
# (the caller re-parses, since every byte offset after it moved).
def reduce_group(file, src, ranges)
  n = 2
  while ranges.size >= 1
    chunk = (ranges.size / n.to_f).ceil
    ranges.each_slice(chunk) do |slice|
      return true if try(file, delete_ranges(src, slice))
    end
    break if chunk == 1
    n = [n * 2, ranges.size].min
  end
  false
end

def statement_pass
  progress = false
  FILES.each do |f|
    next unless File.exist?(f)
    loop do
      src = read(f)
      changed = statement_groups(src).any? { |ranges| reduce_group(f, src, ranges) }
      break unless changed
      progress = true
    end
    warn "#{f}: #{read(f).lines.size} lines (#{$tests} tests)"
  end
  progress
end

HOISTABLE = [Prism::IfNode, Prism::UnlessNode, Prism::WhileNode, Prism::UntilNode, Prism::BeginNode].freeze

def branch_bodies(node)
  case node
  when Prism::IfNode, Prism::UnlessNode
    subsequent = node.respond_to?(:subsequent) ? node.subsequent : node.consequent
    [node.statements, subsequent.respond_to?(:statements) ? subsequent.statements : nil]
  when Prism::WhileNode, Prism::UntilNode, Prism::BeginNode
    [node.statements]
  else
    []
  end.compact
end

def hoist_pass
  progress = false
  FILES.each do |f|
    next unless File.exist?(f)
    loop do
      src = read(f)
      done = false
      stack = [Prism.parse(src).value]
      until stack.empty? || done
        node = stack.pop
        next unless node
        if HOISTABLE.any? { |k| node.is_a?(k) }
          branch_bodies(node).each do |body|
            s, e = line_range(src, node.location)
            text = src.byteslice(body.location.start_offset, body.location.end_offset - body.location.start_offset)
            candidate = src.byteslice(0, s) + text + "\n" + src.byteslice(e, src.bytesize - e)
            if try(f, candidate)
              done = true
              break
            end
          end
        end
        stack.concat(node.compact_child_nodes) unless done
      end
      break unless done
      progress = true
    end
  end
  progress
end

# Blank lines the deletions left behind, squeezed where the result still reproduces.
def tidy_pass
  FILES.each do |f|
    next unless File.exist?(f)
    src = read(f)
    try(f, src.gsub(/\n[ \t]*(?:\n[ \t]*)+\n/, "\n\n").sub(/\A\s+/, ""))
  end
end

abort "the test does not reproduce on the starting files" unless interesting?
loop do
  any = file_pass
  any |= statement_pass
  any |= hoist_pass
  break unless any
end
tidy_pass
warn "done after #{$tests} tests"
FILES.each { |f| puts "#{f}\t#{File.exist?(f) ? read(f).lines.size : 0}" }
