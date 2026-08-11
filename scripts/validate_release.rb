# frozen_string_literal: true

require "json"
require "open3"
require "optparse"

EXPECTED_PACKAGES = %w[favicon_kit_core favicon_kit_web].freeze
VERSION_PATTERN = /(?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*)/.freeze

options = { root: Dir.pwd }
OptionParser.new do |parser|
  parser.on("--root PATH") { |path| options[:root] = path }
  parser.on("--tag TAG") { |tag| options[:tag] = tag }
  parser.on("--notes PATH") { |path| options[:notes] = path }
end.parse!

begin
  root = File.realpath(options.fetch(:root))
  tag = options.fetch(:tag)
  notes_path = options.fetch(:notes)
  match = /\Av(#{VERSION_PATTERN})\z/.match(tag)
  raise "tag must use the vX.Y.Z form" unless match

  version = match[1]
  stdout, stderr, status = Open3.capture3(
    "cargo", "metadata", "--format-version", "1", "--no-deps", "--locked", chdir: root
  )
  raise "cargo metadata failed: #{stderr.strip}" unless status.success?

  metadata = JSON.parse(stdout)
  members = metadata.fetch("workspace_members")
  packages = metadata.fetch("packages").select { |package| members.include?(package.fetch("id")) }
  versions = packages.to_h { |package| [package.fetch("name"), package.fetch("version")] }
  unless versions.keys.sort == EXPECTED_PACKAGES.sort
    raise "workspace release packages must be #{EXPECTED_PACKAGES.join(' and ')}"
  end
  unless versions.values.all? { |package_version| package_version == version }
    details = versions.sort.map { |name, package_version| "#{name}=#{package_version}" }.join(", ")
    raise "workspace package versions do not match #{version}: #{details}"
  end

  changelog = File.read(File.join(root, "CHANGELOG.md"), encoding: "UTF-8")
  header = /^## \[#{Regexp.escape(version)}\] - \d{4}-\d{2}-\d{2}$/
  section = changelog.match(header)
  raise "CHANGELOG.md has no #{version} release section" unless section

  next_section = changelog.match(/^## /, section.end(0))
  boundary = next_section ? next_section.begin(0) : changelog.length
  notes = changelog[section.end(0)...boundary].strip
  raise "CHANGELOG.md #{version} section is empty" if notes.empty?

  File.write(notes_path, "#{notes}\n", encoding: "UTF-8")
rescue JSON::ParserError, KeyError, OptionParser::ParseError, SystemCallError, RuntimeError => error
  warn error.message
  exit 1
end
