# frozen_string_literal: true

require "fileutils"
require "optparse"
require "tmpdir"

REQUIRED_WASM_FILES = %w[
  package.json
  favicon_kit_web.js
  favicon_kit_web_bg.wasm
].freeze
OPTIONAL_WASM_FILES = %w[
  .gitignore
  LICENSE
  LICENSE.md
  README.md
  favicon_kit_web.d.ts
  favicon_kit_web_bg.wasm.d.ts
].freeze
COPIED_WASM_FILES = (REQUIRED_WASM_FILES + OPTIONAL_WASM_FILES).reject do |name|
  name == ".gitignore"
end.freeze

options = {
  root: File.expand_path("..", __dir__),
  wasm_package: nil
}
OptionParser.new do |parser|
  parser.on("--root PATH") { |path| options[:root] = path }
  parser.on("--wasm-package PATH") { |path| options[:wasm_package] = path }
end.parse!

def validate_static_tree(root, current = root)
  Dir.children(current).sort.each do |name|
    path = File.join(current, name)
    metadata = File.lstat(path)
    relative = path.delete_prefix("#{root}/")
    raise "Static UI contains a symbolic link: #{relative}" if metadata.symlink?

    if current == root && name == "pkg"
      raise "Static UI pkg must be a real directory" unless metadata.directory?

      next
    end

    if metadata.directory?
      validate_static_tree(root, path)
    elsif !metadata.file?
      raise "Static UI contains a special file: #{relative}"
    end
  end
end

begin
  root = File.realpath(options[:root])
  source_argument = options[:wasm_package]
  raise "--wasm-package is required" if source_argument.nil? || source_argument.empty?

  source_metadata = File.lstat(source_argument)
  unless source_metadata.directory? && !source_metadata.symlink?
    raise "WASM package must be a real directory"
  end

  source = File.realpath(source_argument)
  source_entries = Dir.children(source).sort
  source_entries.each do |name|
    path = File.join(source, name)
    metadata = File.lstat(path)
    raise "WASM package contains a symbolic link: #{name}" if metadata.symlink?
    raise "WASM package contains a non-file entry: #{name}" unless metadata.file?
    next if (REQUIRED_WASM_FILES + OPTIONAL_WASM_FILES).include?(name)

    raise "WASM package contains an unexpected file: #{name}"
  end
  REQUIRED_WASM_FILES.each do |name|
    raise "WASM package is missing #{name}" unless source_entries.include?(name)
  end

  static = File.join(root, "crates/favicon_kit_web/static")
  static_metadata = File.lstat(static)
  unless static_metadata.directory? && !static_metadata.symlink?
    raise "Static UI must be a real directory"
  end
  validate_static_tree(static)

  index = File.join(static, "index.html")
  index_metadata = File.lstat(index)
  unless index_metadata.file? && !index_metadata.symlink?
    raise "Static UI must contain a real index.html"
  end

  output = File.join(root, "dist")
  if File.exist?(output) || File.symlink?(output)
    output_metadata = File.lstat(output)
    unless output_metadata.directory? && !output_metadata.symlink?
      raise "dist must be a real directory"
    end
  end

  staging = Dir.mktmpdir(".pages-build-", root)
  backup = nil
  begin
    Dir.children(static).sort.each do |name|
      next if name == "pkg"

      FileUtils.copy_entry(File.join(static, name), File.join(staging, name), false, false, true)
    end

    package_output = File.join(staging, "pkg")
    Dir.mkdir(package_output)
    COPIED_WASM_FILES.each do |name|
      source_file = File.join(source, name)
      FileUtils.copy_file(source_file, File.join(package_output, name)) if File.file?(source_file)
    end

    if File.exist?(output)
      backup = Dir.mktmpdir(".pages-previous-", root)
      Dir.rmdir(backup)
      File.rename(output, backup)
    end

    begin
      File.rename(staging, output)
      staging = nil
    rescue StandardError
      File.rename(backup, output) if backup && File.exist?(backup) && !File.exist?(output)
      raise
    end
    FileUtils.remove_entry_secure(backup) if backup && File.exist?(backup)
    puts "Pages artifact assembled in #{output}."
  ensure
    FileUtils.remove_entry_secure(staging) if staging && File.exist?(staging)
  end
rescue OptionParser::ParseError, SystemCallError, RuntimeError => error
  warn error.message
  exit 1
end
