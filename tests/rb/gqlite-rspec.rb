#!/usr/bin/env rspec

require 'rspec'
require 'gqlite'
require 'tempfile'

require_relative 'cucumber/step_definitions/comparison.rb'

def create_node(labels, properties)
  {"type" => "node", "labels" => labels, "properties" => properties}
end

def create_edge(label, properties)
  {"type" => "edge", "labels" => label, "properties" => properties}
end

def create_path(source, labels, properties, destination)
  {"type" => "path", "labels" => labels, "properties" => properties, "source" => source, "destination" => destination}
end

def add_node(list, labels, properties)
  node = create_node(labels, properties)
  list.push node
  return node
end

def add_path(list, labels_1, properties_1, labels_e, properites_e, labels_2, properties_2)
  n1 = create_node(labels_1, properties_1)
  n2 = create_node(labels_2, properties_2)
  list.push [n1, create_edge(labels_e, properites_e), n2, create_path(n1, labels_e, properites_e, n2)]
end

def make_results(gc)
  [["nodes"]] + gc.map { |x| [x]}
end

def make_results_edges(gc)
  [["a", "b", "c", "p"]] + gc
end

def remove_keys(table)
  table.map do |row|
    row.map do |val|
      if val.class == Hash
        val.delete "key"
        if val["type"] == "path"
          val["source"].delete "key"
          val["destination"].delete "key"
        end
      end
      val
    end
  end
end

def get_temp_file()
  ::Dir::Tmpname.create('testdb') do |tmpname, n, opts|
    tmpname
  end
end

RSpec.describe "connection" do
  it "can be created" do
    file = get_temp_file()
    db = GQLite::Connection.new(filename: file)
  end
  it "can be queried with oc to create nodes" do
    file = get_temp_file()
    db = GQLite::Connection.new(filename: file)
    
    # Variable that hold the current state of the graph
    gc = []

    # Test simple create
    db.execute_oc_query "CREATE (n1)"
    add_node gc, [], {}
    nodes = db.execute_oc_query "MATCH (nodes) RETURN nodes"
    expect(remove_keys(nodes)).to eq_in_any_order(make_results(gc))

    # Test create two nodes
    db.execute_oc_query "CREATE (n1), (n2)"
    add_node gc, [], {}
    add_node gc, [], {}
    nodes = db.execute_oc_query "MATCH (nodes) RETURN nodes"
    expect(remove_keys(nodes)).to eq_in_any_order(make_results(gc))
    
    # Test label and return
    n1 = db.execute_oc_query "CREATE (n1:Person) RETURN n1"
    n1_person = add_node(gc, ["Person"], {})
    expect(remove_keys(n1)).to eq([["n1"], [n1_person]])
    nodes = db.execute_oc_query "MATCH (nodes) RETURN nodes"
    expect(remove_keys(nodes)).to eq_in_any_order(make_results(gc))

    # Test two nodes
    db.execute_oc_query "CREATE (n1:Person), (n2:Film)"
    add_node gc, ["Person"], {}
    add_node gc, ["Film"], {}
    nodes = db.execute_oc_query "MATCH (nodes) RETURN nodes"
    expect(remove_keys(nodes)).to eq_in_any_order(make_results(gc))

    # Test properties
    db.execute_oc_query "CREATE (n1 {name: 'Andres', title: 'Developer'})"
    add_node gc, [], {"name" => 'Andres', "title" => 'Developer'}
    nodes = db.execute_oc_query "MATCH (nodes) RETURN nodes"
    expect(remove_keys(nodes)).to eq_in_any_order(make_results(gc))

    # Test label properties
    n1 = db.execute_oc_query "CREATE (n1:Person {name: 'Andres', title: 'Developer'}) RETURN n1"
    n1_ref = add_node gc, ["Person"], {"name" => 'Andres', "title" => 'Developer'}
    expect(remove_keys(n1)).to eq([["n1"],[n1_ref]])
    nodes = db.execute_oc_query "MATCH (nodes) RETURN nodes"
    expect(remove_keys(nodes)).to eq_in_any_order(make_results(gc))
  end
  it "can be queried with oc to create nodes and edges" do
    file = get_temp_file()
    db = GQLite::Connection.new(filename: file)
    
    # Variable that hold the current state of the graph
    gc = []
    gc_edges = []

    # Simple create
    db.execute_oc_query "CREATE (n1), (n2) CREATE (n1)-[:RELTYPE]->(n2)"
    add_node gc, [], {}
    add_node gc, [], {}
    add_path gc_edges, [], {}, ["RELTYPE"], {}, [], {}
    nodes = db.execute_oc_query "MATCH (nodes) RETURN nodes"
    expect(remove_keys(nodes)).to eq_in_any_order(make_results(gc))
    edges = db.execute_oc_query "MATCH (edges)-[]->() RETURN edges"

    # Simple create 2
    db.execute_oc_query "CREATE (n1:cc)-[:RELTYPE]->(n2)"
    add_node gc, ["cc"], {}
    add_node gc, [], {}
    add_path gc_edges, ["cc"], {}, ["RELTYPE"], {}, [], {}
    nodes = db.execute_oc_query "MATCH (nodes) RETURN nodes"
    expect(remove_keys(nodes)).to eq_in_any_order(make_results(gc))

    nodes_edges = db.execute_oc_query "MATCH p = (a)-[b]->(c) RETURN a, b, c, p"
    expect(nodes_edges[1][0]["key"]).to eq(nodes_edges[1][3]["source"]["key"])
    expect(nodes_edges[1][2]["key"]).to eq(nodes_edges[1][3]["destination"]["key"])
    expect(remove_keys(nodes_edges)).to eq_in_any_order(make_results_edges(gc_edges))

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
  it "can compare integers to float" do
    expect(compare(1, 2.0)).to be false
    expect(compare(2, 2.0)).to be true
    expect(compare(2.0, 1)).to be false
    expect(compare(2.0, 2)).to be true
  end
  it "can't compare values of different types" do
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
  it "can compare nodes with keys" do
    expect(compare({"key"=>71399689742590698717330075576608269690, "type" => "node", "labels" => ["a", "b"], "properties" => {}}, {"type" => "node", "labels" => ["b", "a"], "properties" => {}})).to be true
  end
  
  it "can compare nodes with edges" do
    expect(compare({"type"=>"edge", "key"=>248450202975772154401016793053495639615, "labels"=>["T1"], "properties"=>{}}, {"type"=>"edge", "properties"=>{}, "labels"=>["T1"]})).to be true
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

# module OlderDatabaseHelper
#   def validate_db(version)
#     file = get_temp_file()
#     `cp #{__dir__}/data/test-#{version}.db.xz #{file}.xz; xz -d #{file}.xz`
#     db = GQLite::Connection.new(filename: file)
#     nodes = db.execute_oc_query "MATCH (a) RETURN a"
#     edges = db.execute_oc_query "MATCH ()-[a]->() RETURN a"

#     nodes_exp = [ ["a"],
#       [create_node(1, ["A", "E"], {"index"=>0, "name"=>"A"})],
#       [create_node(2, ["B"], {"index"=>1, "name"=>"B"})],
#       [create_node(3, ["C", "F"], {"index"=>2, "name"=>"C"})],
#       [create_node(4, ["D"], {"index"=>3, "name"=>"D"})] ]
#     edges_exp = [ ["a"],
#       [create_edge(1, "AB", {"sum"=>1, "name"=>"AB"})],
#       [create_edge(2, "BC", {"sum"=>3, "name"=>"BC"})],
#       [create_edge(3, "CD", {"sum"=>5, "name"=>"CD"})],
#       [create_edge(4, "DA", {"sum"=>3, "name"=>"DA"})] ]
#     expect(compare_table_in_any_order(nodes_exp, nodes)).to be true
#     expect(compare_table_in_any_order(edges_exp, edges)).to be true

#     nodes_optional = db.execute_oc_query "OPTIONAL MATCH (a:NOT_EXISTING) RETURN a"

#     nodes_optional_exp = [ ["a"], [nil]]

#     expect(compare_table_in_any_order(nodes_optional_exp, nodes_optional)).to be true
#   end
# end

# RSpec.describe "test opening older database" do
#   include OlderDatabaseHelper
#   it "can open 1.0 database" do
#     validate_db("1.0")
#   end
#   it "can open 1.1 database" do
#     validate_db("1.1")
#   end
# end
