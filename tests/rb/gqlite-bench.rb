#!/usr/bin/env ruby

require 'benchmark'
require 'gqlite'
require 'tempfile'

PRNG = Random.new(1234)
FILE = Tempfile.new('testdb')

LABELS = ["A", "B", "C", "D", "E", "F", "G" ]
KEYS = ["KA", "KB", "KC", "KD", "KE", "KF", "KG", "KK", "KF", "KG", "KH"]
VALUES = ["VA", "VB", "VC", "VD", "VE", "VF", "VG", "VK", "VF", "VG", "VH"]

def random_from_array(array)
  array[PRNG.rand(array.size)]
end

def random_label()
  return ":#{random_from_array(LABELS)}"
end

def random_key()
  return random_from_array(KEYS)
end

def random_value()
  return random_from_array(VALUES)
end

def random_labels(sep = "")
  l = ""
  for i in 0...PRNG.rand(5)
    l += sep unless l.empty? 
    l += random_label()
  end
  return l
end

def random_properties()
  p = "{"
  for i in 0...PRNG.rand(5)
    p += ", " if i != 0
    p += "#{random_key()}: \"#{random_value()}\""
  end
  return p + "}"
end

GC.disable

def run_benchmark(create_count, query_count)
  puts "===================================================================="
  puts "= benchmark for #{create_count} relationships #{query_count} queries repeat"
  puts ""
  Benchmark.bm 40 do |x|
    connection = nil
    x.report "INIT" do
      connection = GQLite::Connection.new(filename: FILE.path)
    end
    x.report "CREATE" do
      for i in 0...create_count
        str = "CREATE (#{random_labels()} #{random_properties()})-[#{random_label()} #{random_properties()}]->(#{random_labels()} #{random_properties()})"
        connection.execute_oc_query str
      end
    end
    x.report "MATCH ALL NODES" do
      for i in 0...query_count
        connection.execute_oc_query "MATCH (n) RETURN n"
      end
    end
    x.report "MATCH ALL EDGES" do
      for i in 0...query_count
        connection.execute_oc_query "MATCH ()-[r]->() RETURN r"
      end
    end
    x.report "MATCH NODES/LABEL" do
      for l in LABELS
        connection.execute_oc_query "MATCH (n:#{l}) RETURN n"
      end
    end
    x.report "MATCH NODES/MANY LABELS" do
      for i in 0...query_count
        connection.execute_oc_query "MATCH (n#{random_labels()}) RETURN n"
      end
    end
    x.report "MATCH EDGES/LABEL" do
      for l in LABELS
        connection.execute_oc_query "MATCH ()-[r:#{l}]->() RETURN r"
      end
    end
    x.report "MATCH EDGES/MANY LABELS" do
      for i in 0...query_count
        connection.execute_oc_query "MATCH ()-[r#{random_labels("|")}]->() RETURN r"
      end
    end
    x.report "MATCH EDGES/NODES LABEL" do
      for l in LABELS
        connection.execute_oc_query "MATCH (:#{l})-[r]->() RETURN r"
      end
    end
    x.report "MATCH EDGES/NODES MANY LABELS" do
      for i in 0...query_count
        connection.execute_oc_query "MATCH (#{random_labels()})-[r]->() RETURN r"
      end
    end
    x.report "MATCH NODES/PROPERTIES" do
      for k in KEYS
        for v in VALUES
          connection.execute_oc_query "MATCH (n {#{k}: \"#{v}\"}) RETURN n"
        end
      end
    end
    x.report "MATCH EDGES/PROPERTIES" do
      for k in KEYS
        for v in VALUES
          connection.execute_oc_query "MATCH ()-[r {#{k}: \"#{v}\"}]->() RETURN r"
        end
      end
    end
    x.report "MATCH NODES/LABELS+PROPERTIES" do
      for k in KEYS
        for v in VALUES
          for i in 0...query_count
            connection.execute_oc_query "MATCH (n#{random_labels()} {#{k}: \"#{v}\"}) RETURN n"
          end
        end
      end
    end
    x.report "MATCH EDGES/LABELS+PROPERTIES" do
      for k in KEYS
        for v in VALUES
          for i in 0...query_count
            connection.execute_oc_query "MATCH ()-[r#{random_labels('|')} {#{k}: \"#{v}\"}]->() RETURN r"
          end
        end
      end
    end
    x.report "MATCH EDGES/NODES LABELS+PROPERTIES" do
      for k in KEYS
        for v in VALUES
          for i in 0...query_count
            connection.execute_oc_query "MATCH (#{random_labels()} {#{k}: \"#{v}\"})-[r]->() RETURN r"
          end
        end
      end
    end
    x.report "MATCH EDGES NODES / LABELS+PROPERTIES" do
      for kn in KEYS
        for vn in VALUES
          for ke in KEYS
            for ve in VALUES
              for i in 0...query_count
                connection.execute_oc_query "MATCH (#{random_labels()} {#{kn}: \"#{vn}\"})-[r#{random_labels('|')} {#{ke}: \"#{ve}\"}]->() RETURN r"
              end
            end
          end
        end
      end
    end
    x.report "MATCH MATCH NODES" do
      connection.execute_oc_query "MATCH (n) MATCH (m) WHERE labels(n) = labels(m) RETURN n,m"
    end
    x.report "MATCH MATCH EDGES" do
      connection.execute_oc_query "MATCH ()-[r]->() MATCH ()-[t]->() WHERE type(r) = type(t) RETURN r,t"
    end
  end
end

run_benchmark(10, 20)
run_benchmark(100, 20)
run_benchmark(1000, 10)
run_benchmark(10000, 5)
