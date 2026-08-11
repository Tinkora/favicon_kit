# frozen_string_literal: true

require "fileutils"
require "minitest/autorun"
require "open3"
require "rbconfig"
require "tmpdir"

class AssemblePagesTest < Minitest::Test
  ASSEMBLER = File.expand_path("assemble_pages.rb", __dir__)

  def test_assembles_the_web_ui_with_the_verified_wasm_package
    with_fixture do |root, wasm_package|
      result = run_assembler(root, wasm_package)

      assert result[:status].success?, result[:output]
      assert_equal "<!doctype html>\n<title>Favicon Kit</title>\n", read(root, "dist/index.html")
      assert_equal "body {}\n", read(root, "dist/styles.css")
      assert_equal "export default {};\n", read(root, "dist/pkg/favicon_kit_web.js")
      assert File.file?(File.join(root, "dist/pkg/favicon_kit_web_bg.wasm"))
      refute File.exist?(File.join(root, "dist/pkg/.gitignore"))
      refute File.exist?(File.join(root, "dist/sentinel.txt"))
    end
  end

  def test_rejects_an_unexpected_wasm_file_without_replacing_the_previous_site
    with_fixture do |root, wasm_package|
      File.write(File.join(wasm_package, "unexpected.txt"), "no\n", encoding: "UTF-8")

      result = run_assembler(root, wasm_package)

      refute result[:status].success?
      assert_includes result[:output], "unexpected file"
      assert_equal "previous\n", read(root, "dist/sentinel.txt")
    end
  end

  def test_rejects_a_symbolic_link_in_the_static_ui
    with_fixture do |root, wasm_package|
      File.symlink("index.html", File.join(root, "crates/favicon_kit_web/static/linked.html"))

      result = run_assembler(root, wasm_package)

      refute result[:status].success?
      assert_includes result[:output], "Static UI contains a symbolic link"
      assert_equal "previous\n", read(root, "dist/sentinel.txt")
    end
  end

  private

  def with_fixture
    Dir.mktmpdir("assemble-pages-") do |root|
      static = File.join(root, "crates/favicon_kit_web/static")
      wasm_package = File.join(root, "wasm-package")
      FileUtils.mkdir_p(static)
      FileUtils.mkdir_p(wasm_package)
      FileUtils.mkdir_p(File.join(root, "dist"))
      File.write(File.join(root, "dist/sentinel.txt"), "previous\n", encoding: "UTF-8")
      File.write(
        File.join(static, "index.html"),
        "<!doctype html>\n<title>Favicon Kit</title>\n",
        encoding: "UTF-8"
      )
      File.write(File.join(static, "styles.css"), "body {}\n", encoding: "UTF-8")
      FileUtils.mkdir_p(File.join(static, "pkg"))
      File.write(File.join(static, "pkg/stale.js"), "stale\n", encoding: "UTF-8")
      File.write(File.join(wasm_package, "package.json"), "{}\n", encoding: "UTF-8")
      File.write(
        File.join(wasm_package, "favicon_kit_web.js"),
        "export default {};\n",
        encoding: "UTF-8"
      )
      File.binwrite(File.join(wasm_package, "favicon_kit_web_bg.wasm"), "\0asm")
      File.write(File.join(wasm_package, ".gitignore"), "*\n", encoding: "UTF-8")
      yield root, wasm_package
    end
  end

  def run_assembler(root, wasm_package)
    stdout, stderr, status = Open3.capture3(
      RbConfig.ruby,
      ASSEMBLER,
      "--root",
      root,
      "--wasm-package",
      wasm_package
    )
    { output: stdout + stderr, status: status }
  end

  def read(root, path)
    File.read(File.join(root, path), encoding: "UTF-8")
  end
end
