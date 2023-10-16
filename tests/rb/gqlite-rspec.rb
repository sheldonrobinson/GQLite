#!/usr/bin/env rspec

require 'rspec'
require 'gqlite'
require 'tempfile'

require_relative 'cucumber/step_definitions/comparison.rb'

def create_node(id, labels, properties)
  {"type" => "node", "id" => id, "labels" => labels, "properties" => properties}
end

def create_edge(id, label, properties)
  {"type" => "edge", "id" => id, "label" => label, "properties" => properties}
end

def add_node(list, id, labels, properties)
  node = create_node(id, labels, properties)
  list.push node
  return node
end

def add_edge(list, id_1, labels_1, properties_1, id_e, label_e, properites_e, id_2, labels_2, properties_2)
  list.push [create_node(id_1, labels_1, properties_1), create_edge(id_e, label_e, properites_e), create_node(id_2, labels_2, properties_2)]
end

def make_results(gc)
  [["nodes"]] + gc.map { |x| [x]}
end

def make_results_edges(gc)
  [["a", "b", "c"]] + gc
end

RSpec.describe "connection" do
  it "can be created" do
    file = Tempfile.new('testdb')
    db = GQLite::Connection.new(sqlite_filename: file.path)
  end
  it "can be queried with oc to create nodes" do
    file = Tempfile.new('testdb')
    db = GQLite::Connection.new(sqlite_filename: file.path)
    
    # Variable that hold the current state of the graph
    gc = []

    # Test simple create
    db.execute_oc_query "CREATE (n1)"
    add_node gc, 1, [], {}
    nodes = db.execute_oc_query "MATCH (nodes) RETURN nodes"
    expect(nodes).to eq(make_results(gc))

    # Test create two nodes
    db.execute_oc_query "CREATE (n1), (n2)"
    add_node gc, 2, [], {}
    add_node gc, 3, [], {}
    nodes = db.execute_oc_query "MATCH (nodes) RETURN nodes"
    expect(nodes).to eq(make_results(gc))
    
    # Test label and return
    n1 = db.execute_oc_query "CREATE (n1:Person) RETURN n1"
    n1_person = add_node(gc, 4, ["Person"], {})
    expect(n1).to eq([["n1"], [n1_person]])
    nodes = db.execute_oc_query "MATCH (nodes) RETURN nodes"
    expect(nodes).to eq(make_results(gc))

    # Test two nodes
    db.execute_oc_query "CREATE (n1:Person), (n2:Film)"
    add_node gc, 5, ["Person"], {}
    add_node gc, 6, ["Film"], {}
    nodes = db.execute_oc_query "MATCH (nodes) RETURN nodes"
    expect(nodes).to eq(make_results(gc))

    # Test properties
    db.execute_oc_query "CREATE (n1 {name: 'Andres', title: 'Developer'})"
    add_node gc, 7, [], {"name" => 'Andres', "title" => 'Developer'}
    nodes = db.execute_oc_query "MATCH (nodes) RETURN nodes"
    expect(nodes).to eq(make_results(gc))

    # Test label properties
    n1 = db.execute_oc_query "CREATE (n1:Person {name: 'Andres', title: 'Developer'}) RETURN n1"
    n1_ref = add_node gc, 8, ["Person"], {"name" => 'Andres', "title" => 'Developer'}
    expect(n1).to eq([["n1"],[n1_ref]])
    nodes = db.execute_oc_query "MATCH (nodes) RETURN nodes"
    expect(nodes).to eq(make_results(gc))
  end
  it "can be queried with oc to create nodes and edges" do
    file = Tempfile.new('testdb')
    db = GQLite::Connection.new(sqlite_filename: file.path)
    
    # Variable that hold the current state of the graph
    gc = []
    gc_edges = []

    # Simple create
    db.execute_oc_query "CREATE (n1), (n2) CREATE (n1)-[:RELTYPE]->(n2)"
    add_node gc, 1, [], {}
    add_node gc, 2, [], {}
    add_edge gc_edges, 1, [], {}, 1, "RELTYPE", {}, 2, [], {}
    nodes = db.execute_oc_query "MATCH (nodes) RETURN nodes"
    expect(nodes).to eq(make_results(gc))
    edges = db.execute_oc_query "MATCH (edges)-[]->() RETURN edges"

    # Simple create 2
    db.execute_oc_query "CREATE (n1)-[:RELTYPE]->(n2)"
    add_node gc, 3, [], {}
    add_node gc, 4, [], {}
    add_edge gc_edges, 3, [], {}, 2, "RELTYPE", {}, 4, [], {}
    nodes = db.execute_oc_query "MATCH (nodes) RETURN nodes"
    expect(nodes).to eq(make_results(gc))

    nodes_edges = db.execute_oc_query "MATCH (a)-[b]->(c) RETURN a, b, c"
    expect(nodes_edges).to eq(make_results_edges(gc_edges))

    # Create with match
    db.execute_oc_query "CREATE (:X), (:Y)"
    db.execute_oc_query "MATCH (x:X), (y:Y) CREATE (x)-[:R]->(y)"

    # p = db.execute_oc_query "CREATE p = (andres {name:'Andres'})-[:WORKS_AT]->(neo)<-[:WORKS_AT]-(michael {name: 'Michael'}) RETURN p"
  end
end

RSpec.describe "compare" do
  it "can compare basic types" do
    expect(compare(1, 2)).to be false
    expect(compare(2, 2)).to be true
    expect(compare(2.001, 2.001)).to be true
    expect(compare(2.0000001, 2.0000002)).to be false
    expect(compare(2.0000000000001, 2.0000000000002)).to be true
    expect(compare("abc", "abc")).to be true
    expect(compare("abc", "abcd")).to be false
    expect(compare("abcd", "abc")).to be false
    expect(compare("abd", "abc")).to be false
  end
  it "can't compare values of different types" do
    expect(compare(1, 2.0)).to be false
    expect(compare(2, 2.0)).to be false
    expect(compare(2.0, 1)).to be false
    expect(compare(2.0, 2)).to be false
    expect(compare("1", 1)).to be false
    expect(compare("1.0", 1.0)).to be false
    expect(compare(1.0, "1.0")).to be false
    expect(compare(1, "1")).to be false
    expect(compare(1, [1])).to be false
    expect(compare(1.0, [1.0])).to be false
    expect(compare("a", ["a"])).to be false
    expect(compare([1], 1)).to be false
    expect(compare([1.0], 1.0)).to be false
    expect(compare(["a"], "a")).to be false
    expect(compare(1, {"k" => 1})).to be false
    expect(compare(1.0, {"k" => 1.0})).to be false
    expect(compare("a", {"k" => "a"})).to be false
    expect(compare([1, 1.0, "a"], {"k" => [1, 1.0, "a"]})).to be false
    expect(compare({"k" => 1}, 1)).to be false
    expect(compare({"k" => 1.0}, 1.0)).to be false
    expect(compare({"k" => "a"}, "a")).to be false
    expect(compare({"k" => [1, 1.0, "a"]}, [1, 1.0, "a"])).to be false
  end
  it "can compare arrays" do
    expect(compare([1], [2])).to be false
    expect(compare([1, 1.0], [1, 1.0])).to be true
    expect(compare([1, 1.0], [1, 2.0])).to be false
    expect(compare([1, 1.0, "a"], [1, 1.0, "a"])).to be true
    expect(compare([1, 1.0, "a"], [1, 1.0, "b"])).to be false
    expect(compare([1, 1.0, ["a"]], [1, 1.0, ["b"]])).to be false
    expect(compare([1, 1.0, ["a"]], [1, 1.0, ["a"]])).to be true
    expect(compare([1, 1.0, ["a"]], [1, 1.0, ["a", 1]])).to be false
    expect(compare([1, 1.0, ["a"]], [1, 1.0])).to be false
  end
  it "can compare hashes" do
    expect(compare({"k" => 1}, {"k" => 2})).to be false
    expect(compare({"k" => 1}, {"k" => 1})).to be true
    expect(compare({"k" => [1]}, {"k" => [1]})).to be true
    expect(compare({"k" => {"k" => [1]}}, {"k" => {"k" => [1]}})).to be true
    expect(compare({"k" => {"k" => [1]}}, {"k" => {"k" => [2]}})).to be false
    expect(compare({"k" => {"k" => [1]}}, {"k" => {"k2" => [1]}})).to be false
    expect(compare({"k" => {"k" => [1]}}, {"k2" => {"k" => [1]}})).to be false
  end
  it "can compare nodes with labels in different order" do
    expect(compare({"type" => "node", "labels" => ["a", "b"], "properties" => {}}, {"type" => "node", "labels" => ["b", "a"], "properties" => {}})).to be true
    expect(compare({"type" => "node", "labels" => ["a", "b"], "properties" => {}}, {"type" => "node", "labels" => ["b", "c"], "properties" => {}})).to be false
    expect(compare({"type" => "node", "labels" => ["a", "b"], "properties" => {}}, {"type" => "node", "labels" => ["a", "b"], "properties" => {"k" => 1}})).to be false
    expect(compare({"type" => "node", "labels" => ["a", "b"], "properties" => {"k" => 1}}, {"type" => "node", "labels" => ["b", "a"], "properties" => {"k" => 1}})).to be true
  end
end

RSpec.describe "compare tables" do
  it "works" do
    expect(compare_table_in_any_order([["p"], ["foo"]], [["p"], ["foo"]])).to be true
    expect(compare_table_in_any_order([["p"], ["foo"]], [["p"], ["fo0"]])).to be false
    expect(compare_table_in_any_order([["p"], ["foo"]], [["p"]])).to be false
    expect(compare_table_in_any_order([["p"]], [["p"], ["foo"]])).to be false
    expect(compare_table_in_any_order([["a", "b", "result"], [1, 1, true], [1, 0, true], [0, 1, true], [0, 0, true]],
                                      [["a", "b", "result"], [true, true, true], [true, false, true], [false, true, true], [false, false, true]])).to be true
  end
  it "in any column order" do
    expect(compare_table_in_any_order([["p", "a"], ["foo", 1]], [["a", "p"], [1, "foo"]])).to be true
    expect(compare_table_in_any_order([["a", "p"], [1, "foo"]], [["p", "a"], ["foo", 1]])).to be true
    expect(compare_table_in_any_order([["b", "p"], [1, "foo"]], [["p", "a"], ["foo", 1]])).to be false
  end
end
