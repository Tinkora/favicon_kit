# frozen_string_literal: true

require "fileutils"
require "minitest/autorun"
require "open3"
require "rbconfig"
require "tmpdir"
require "yaml"

class CheckWorkflowContractsTest < Minitest::Test
  CHECKER = File.expand_path("check_workflow_contracts.rb", __dir__)
  COMMIT = "21145ce218263e3b30359bab0c748da4702f801b"

  def test_accepts_the_required_tinkora_workflow_calls
    with_fixture do |root|
      result = run_checker(root)

      assert result[:status].success?, result[:output]
      assert_includes result[:output], "Workflow contracts passed"
    end
  end

  def test_rejects_a_floating_reusable_workflow_reference
    with_fixture(reference: "main") do |root|
      result = run_checker(root)

      refute result[:status].success?
      assert_includes result[:output], "must be pinned"
    end
  end

  def test_rejects_a_missing_msrv_check
    with_fixture(msrv: "") do |root|
      result = run_checker(root)

      refute result[:status].success?
      assert_includes result[:output], "must set msrv to 1.95.0"
    end
  end

  def test_rejects_pages_that_do_not_publish_the_web_ui
    with_fixture(pages_path: "index.html") do |root|
      result = run_checker(root)

      refute result[:status].success?
      assert_includes result[:output], "must assemble crates/favicon_kit_web/static"
    end
  end

  def test_rejects_a_disabled_playwright_smoke_check
    with_fixture(playwright_smoke: false) do |root|
      result = run_checker(root)

      refute result[:status].success?
      assert_includes result[:output], "must enable Playwright smoke"
    end
  end

  private

  def with_fixture(reference: COMMIT, msrv: "1.95.0", pages_path: "crates/favicon_kit_web/static", playwright_smoke: true)
    Dir.mktmpdir("workflow-contracts-") do |root|
      write_workflows(root, reference, msrv, pages_path, playwright_smoke)
      yield root
    end
  end

  def write_workflows(root, reference, msrv, pages_path, playwright_smoke)
    write_yaml(root, ".github/workflows/quality.yml", {
      "jobs" => {
        "rust" => {
          "uses" => "Tinkora/.github/.github/workflows/reusable-rust-quality.yml@#{reference}",
          "with" => { "toolchain" => "1.95.0", "locked" => true, "msrv" => msrv, "coverage" => true }
        },
        "wasm" => {
          "uses" => "Tinkora/.github/.github/workflows/reusable-wasm-quality.yml@#{reference}",
          "with" => {
            "working-directory" => "crates/favicon_kit_web",
            "toolchain" => "1.95.0",
            "locked" => true,
            "playwright-smoke" => playwright_smoke,
            "node-version" => "24"
          }
        }
      }
    })
    write_yaml(root, ".github/workflows/supply-chain.yml", {
      "jobs" => {
        "audit" => {
          "uses" => "Tinkora/.github/.github/workflows/reusable-supply-chain.yml@#{reference}",
          "with" => { "toolchain" => "1.95.0" }
        }
      }
    })
    write_yaml(root, ".github/workflows/pages.yml", {
      "jobs" => {
        "assemble" => {
          "if" => "github.ref == 'refs/heads/main'",
          "needs" => %w[quality documentation supply-chain],
          "steps" => [
            { "run" => "ruby scripts/assemble_pages.rb --source #{pages_path}" },
            { "with" => { "name" => "wasm-package-${{ github.run_id }}-${{ github.run_attempt }}" } },
            { "with" => { "name" => "pages-source-${{ github.run_id }}-${{ github.run_attempt }}" } }
          ]
        },
        "deploy" => {
          "if" => "github.ref == 'refs/heads/main'",
          "uses" => "Tinkora/.github/.github/workflows/reusable-pages.yml@#{reference}",
          "with" => { "source-artifact-name" => "pages-source-${{ github.run_id }}-${{ github.run_attempt }}" }
        }
      }
    })
    write_yaml(root, ".github/workflows/release.yml", {
      "jobs" => {
        "evidence" => {
          "uses" => "Tinkora/.github/.github/workflows/reusable-release.yml@#{reference}",
          "with" => { "publish" => false }
        }
      }
    })
  end

  def write_yaml(root, path, document)
    absolute_path = File.join(root, path)
    FileUtils.mkdir_p(File.dirname(absolute_path))
    File.write(absolute_path, YAML.dump(document), encoding: "UTF-8")
  end

  def run_checker(root)
    stdout, stderr, status = Open3.capture3(RbConfig.ruby, CHECKER, "--root", root)
    { output: stdout + stderr, status: status }
  end
end
