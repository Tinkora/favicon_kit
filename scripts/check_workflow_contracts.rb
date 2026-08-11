# frozen_string_literal: true

require "optparse"
require "yaml"

REUSABLE_WORKFLOW_COMMIT = "21145ce218263e3b30359bab0c748da4702f801b"
RUST_VERSION = "1.95.0"
PAGES_MAIN_CONDITION = "github.ref == 'refs/heads/main'"
PAGES_WASM_ARTIFACT = "wasm-package-${{ github.run_id }}-${{ github.run_attempt }}"
PAGES_SOURCE_ARTIFACT = "pages-source-${{ github.run_id }}-${{ github.run_attempt }}"
EXPECTED_CALLS = {
  ".github/workflows/quality.yml" => {
    "rust" => "Tinkora/.github/.github/workflows/reusable-rust-quality.yml@#{REUSABLE_WORKFLOW_COMMIT}",
    "wasm" => "Tinkora/.github/.github/workflows/reusable-wasm-quality.yml@#{REUSABLE_WORKFLOW_COMMIT}"
  },
  ".github/workflows/supply-chain.yml" => {
    "audit" => "Tinkora/.github/.github/workflows/reusable-supply-chain.yml@#{REUSABLE_WORKFLOW_COMMIT}"
  },
  ".github/workflows/pages.yml" => {
    "deploy" => "Tinkora/.github/.github/workflows/reusable-pages.yml@#{REUSABLE_WORKFLOW_COMMIT}"
  },
  ".github/workflows/release.yml" => {
    "evidence" => "Tinkora/.github/.github/workflows/reusable-release.yml@#{REUSABLE_WORKFLOW_COMMIT}"
  }
}.freeze

options = { root: Dir.pwd }
OptionParser.new do |parser|
  parser.on("--root PATH") { |path| options[:root] = path }
end.parse!

def string_values(value)
  case value
  when Hash
    value.values.flat_map { |child| string_values(child) }
  when Array
    value.flat_map { |child| string_values(child) }
  when String
    [value]
  else
    []
  end
end

def action_references(value)
  case value
  when Hash
    references = value.filter_map { |key, child| child if key == "uses" && child.is_a?(String) }
    references + value.values.flat_map { |child| action_references(child) }
  when Array
    value.flat_map { |child| action_references(child) }
  else
    []
  end
end

root = File.expand_path(options[:root])
errors = []
workflows = {}

Dir.glob(File.join(root, ".github/workflows/*.{yml,yaml}")).sort.each do |path|
  relative_path = path.delete_prefix("#{root}/")
  begin
    workflow = YAML.safe_load_file(path, aliases: false)
    workflows[relative_path] = workflow
    action_references(workflow).each do |reference|
      next if reference.start_with?("./")
      next if reference.match?(/\A[^@\s]+@[0-9a-f]{40}\z/)

      errors << "#{relative_path} action #{reference} must be pinned to a full commit SHA"
    end
  rescue Psych::Exception => error
    errors << "Invalid workflow #{relative_path}: #{error.message}"
  end
end

EXPECTED_CALLS.each do |relative_path, expected_jobs|
  workflow = workflows[relative_path]
  unless workflow
    errors << "Missing workflow: #{relative_path}"
    next
  end

  jobs = workflow.fetch("jobs", {})
  expected_jobs.each do |job_name, expected_reference|
    actual_reference = jobs.dig(job_name, "uses")
    next if actual_reference == expected_reference

    errors << "#{relative_path} job #{job_name} must use #{expected_reference}"
  end
end

quality_jobs = workflows.dig(".github/workflows/quality.yml", "jobs") || {}
rust_inputs = quality_jobs.dig("rust", "with") || {}
wasm_inputs = quality_jobs.dig("wasm", "with") || {}
errors << ".github/workflows/quality.yml must set Rust toolchain to #{RUST_VERSION}" unless rust_inputs["toolchain"] == RUST_VERSION
errors << ".github/workflows/quality.yml must set msrv to #{RUST_VERSION}" unless rust_inputs["msrv"] == RUST_VERSION
errors << ".github/workflows/quality.yml must enable locked Rust builds" unless rust_inputs["locked"] == true
errors << ".github/workflows/quality.yml must enable coverage" unless rust_inputs["coverage"] == true
unless wasm_inputs["working-directory"] == "crates/favicon_kit_web"
  errors << ".github/workflows/quality.yml must build crates/favicon_kit_web"
end
errors << ".github/workflows/quality.yml must set WASM toolchain to #{RUST_VERSION}" unless wasm_inputs["toolchain"] == RUST_VERSION
errors << ".github/workflows/quality.yml must enable locked WASM builds" unless wasm_inputs["locked"] == true
errors << ".github/workflows/quality.yml must enable Playwright smoke" unless wasm_inputs["playwright-smoke"] == true
errors << ".github/workflows/quality.yml must use Node 24 for Playwright smoke" unless wasm_inputs["node-version"] == "24"

supply_inputs = workflows.dig(".github/workflows/supply-chain.yml", "jobs", "audit", "with") || {}
unless supply_inputs["toolchain"] == RUST_VERSION
  errors << ".github/workflows/supply-chain.yml must set toolchain to #{RUST_VERSION}"
end

pages_jobs = workflows.dig(".github/workflows/pages.yml", "jobs") || {}
unless %w[assemble deploy].all? { |job_name| pages_jobs.dig(job_name, "if") == PAGES_MAIN_CONDITION }
  errors << ".github/workflows/pages.yml must restrict assembly and deployment to main"
end
unless Array(pages_jobs.dig("assemble", "needs")).sort == %w[documentation quality supply-chain]
  errors << ".github/workflows/pages.yml assembly must wait for quality, documentation, and supply-chain"
end
pages_values = string_values(pages_jobs)
unless pages_values.include?(PAGES_WASM_ARTIFACT) && pages_values.count(PAGES_SOURCE_ARTIFACT) >= 2
  errors << ".github/workflows/pages.yml artifact names must include github.run_attempt"
end
unless pages_values.any? { |value| value.include?("crates/favicon_kit_web/static") }
  errors << ".github/workflows/pages.yml must assemble crates/favicon_kit_web/static"
end

release_inputs = workflows.dig(".github/workflows/release.yml", "jobs", "evidence", "with") || {}
unless release_inputs["publish"] == false
  errors << ".github/workflows/release.yml reusable release must remain dry-run"
end

if errors.empty?
  puts "Workflow contracts passed (Tinkora bundle #{REUSABLE_WORKFLOW_COMMIT})."
  exit 0
end

errors.each { |error| warn error }
exit 1
