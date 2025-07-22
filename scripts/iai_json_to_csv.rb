require 'json'
require 'csv'

# Helper to safely extract integer metric
def get_metric(callgrind, key)
  value = callgrind.dig(key, "metrics", "Left", "Int")
  value.is_a?(Integer) ? value : nil
end

# Main execution
rows = []

ARGV.each do |file_path|
  begin
    json = JSON.parse(File.read(file_path))
    group = json["function_name"]
    backend = json["id"]

    callgrind = json.dig("profiles", 0, "summaries", "total", "summary", "Callgrind")
    next unless callgrind

    row = [
      group,
      backend,
      get_metric(callgrind, "Ir"),
      get_metric(callgrind, "L1hits"),
      get_metric(callgrind, "LLhits"),
      get_metric(callgrind, "RamHits"),
      get_metric(callgrind, "TotalRW"),
      get_metric(callgrind, "EstimatedCycles")
    ]

    rows << row unless row.any?(&:nil?)
  rescue => e
    STDERR.puts "Error processing #{file_path}: #{e.message}"
  end
end

# Output CSV header + rows
CSV($stdout) do |csv|
  csv << ["group", "backend", "Ir", "L1hits", "LLhits", "RamHits", "TotalRW", "EstimatedCycles"]
  rows.each { |row| csv << row }
end
