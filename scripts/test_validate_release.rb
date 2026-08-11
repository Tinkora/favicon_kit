# frozen_string_literal: true

require "fileutils"
require "minitest/autorun"
require "open3"
require "rbconfig"
require "tmpdir"

class ValidateReleaseTest < Minitest::Test
  VALIDATOR = File.expand_path("validate_release.rb", __dir__)

  def test_accepts_a_tag_matching_both_crates_and_a_nonempty_changelog_section
    with_fixture do |root|
      notes = File.join(root, "release-notes.md")
      result = run_validator(root, "v1.2.3", notes)

      assert result[:status].success?, result[:output]
      assert_equal "### Added\n- Initial release\n", File.read(notes, encoding: "UTF-8")
    end
  end

  def test_rejects_a_tag_that_does_not_match_the_two_crate_versions
    with_fixture(web_version: "1.2.4") do |root|
      result = run_validator(root, "v1.2.3", File.join(root, "release-notes.md"))

      refute result[:status].success?
      assert_includes result[:output], "versions do not match 1.2.3"
    end
  end

  def test_rejects_an_empty_changelog_section
    with_fixture(changelog_notes: "") do |root|
      result = run_validator(root, "v1.2.3", File.join(root, "release-notes.md"))

      refute result[:status].success?
      assert_includes result[:output], "section is empty"
    end
  end

  private

  def with_fixture(web_version: "1.2.3", changelog_notes: "### Added\n- Initial release\n")
    Dir.mktmpdir("validate-release-") do |root|
      write_workspace(root, web_version, changelog_notes)
      generate_lockfile(root)
      yield root
    end
  end

  def write_workspace(root, web_version, changelog_notes)
    FileUtils.mkdir_p(File.join(root, "crates/favicon_kit_core"))
    FileUtils.mkdir_p(File.join(root, "crates/favicon_kit_web"))
    File.write(
      File.join(root, "Cargo.toml"),
      <<~TOML,
        [workspace]
        members = ["crates/favicon_kit_core", "crates/favicon_kit_web"]
        resolver = "3"
      TOML
      encoding: "UTF-8"
    )
    write_package(root, "favicon_kit_core", "1.2.3")
    write_package(root, "favicon_kit_web", web_version)
    File.write(
      File.join(root, "CHANGELOG.md"),
      "# Changelog\n\n## [1.2.3] - 2026-08-12\n\n#{changelog_notes}\n## [0.1.0] - 2026-08-01\n\n- Previous\n",
      encoding: "UTF-8"
    )
  end

  def write_package(root, name, version)
    FileUtils.mkdir_p(File.join(root, "crates", name, "src"))
    File.write(
      File.join(root, "crates", name, "Cargo.toml"),
      <<~TOML,
        [package]
        name = "#{name}"
        version = "#{version}"
        edition = "2024"
      TOML
      encoding: "UTF-8"
    )
    File.write(File.join(root, "crates", name, "src/lib.rs"), "", encoding: "UTF-8")
  end

  def generate_lockfile(root)
    _stdout, stderr, status = Open3.capture3("cargo", "generate-lockfile", chdir: root)
    assert status.success?, stderr
  end

  def run_validator(root, tag, notes)
    stdout, stderr, status = Open3.capture3(
      RbConfig.ruby,
      VALIDATOR,
      "--root",
      root,
      "--tag",
      tag,
      "--notes",
      notes
    )
    { output: stdout + stderr, status: status }
  end
end
