# Lines up Rails' route table (`rails_routes.rb`) with roundhouse's
# (`dump_routes`) and prints where they differ:
#   ruby compare.rb rails.txt roundhouse.txt
#
# A route is VERB, PATH (segment names ignored: `:id` and `:post_id` are
# one shape) and controller#action. Order matters only among routes of
# the same verb and shape, where the first wins. Rails names a route
# once, on its first line; names are compared as name -> path.

Route = Struct.new(:verb, :path, :endpoint, :name, :state) do
  def shape = path.gsub(/:[a-z_0-9]+/, ":").gsub(/\*[a-z_0-9]+/, "*")
  def key = [verb, shape]
end

def read(file)
  File.readlines(file, chomp: true).filter_map do |line|
    verb, path, endpoint, name, state = line.split(" ")
    Route.new(verb, path, endpoint, name, state || "dispatched") if verb
  end
end

rails = read(ARGV.fetch(0)).reject { |r| %w[INTERNAL ENGINE RACK].include?(r.endpoint) }
ours = read(ARGV.fetch(1))
served = ours.reject { |r| r.state.start_with?("refused") }

first = ->(routes) { routes.each_with_object({}) { |r, h| h[r.key] ||= r } }
rails_first, ours_first = first.(rails), first.(served)

missing = rails_first.reject { |k, _| ours_first.key?(k) }.values
extra = ours_first.reject { |k, _| rails_first.key?(k) }.values
wrong = rails_first.filter_map { |k, r| (o = ours_first[k]) && o.endpoint != r.endpoint ? [r, o] : nil }
refused = ours.select { |r| r.state.start_with?("refused") }.group_by(&:state).transform_values(&:size)

rails_names = rails.reject { |r| r.name == "-" }.each_with_object({}) { |r, h| h[r.name] ||= r.shape }
our_names = ours.reject { |r| r.name == "-" }.each_with_object({}) { |r, h| h[r.name] ||= r.shape }
names_missing = rails_names.keys - our_names.keys
names_moved = rails_names.select { |n, s| our_names[n] && our_names[n] != s }.keys

puts "rails #{rails_first.size} routes, roundhouse #{ours_first.size} served (#{refused.values.sum} refused: #{refused.map { |k, v| "#{k.sub("refused:", "")} #{v}" }.join(", ")})"
puts "missing #{missing.size}, extra #{extra.size}, different endpoint #{wrong.size}, helpers missing #{names_missing.size}, helpers at another path #{names_moved.size}"
puts "\n## missing (Rails serves, roundhouse does not)"; missing.each { |r| puts "  #{r.verb} #{r.path} #{r.endpoint}#{ours.any? { |o| o.key == r.key } ? "  [refused here]" : ""}" }
puts "\n## extra (roundhouse serves, Rails does not)"; extra.each { |r| puts "  #{r.verb} #{r.path} #{r.endpoint}" }
puts "\n## different endpoint (rails -> roundhouse)"; wrong.each { |r, o| puts "  #{r.verb} #{r.path} #{r.endpoint} -> #{o.endpoint}" }
puts "\n## helpers missing"; names_missing.each { |n| puts "  #{n} #{rails_names[n]}" }
puts "\n## helpers at another path"; names_moved.each { |n| puts "  #{n} rails #{rails_names[n]} roundhouse #{our_names[n]}" }
