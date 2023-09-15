#!/usr/bin/env rspec

require 'rspec'
require 'gqlite'
require 'tempfile'

def create_node(id, labels, properties)
  {"type" => "node", "id" => id, "labels" => labels, "properties" => properties}
end

def add_node(list, id, labels, properties)
  node = create_node(id, labels, properties)
  list.push node
  return node
end

def make_results(gc)
  [["nodes"]] + gc.map { |x| [x]}
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
    nodes = db.execute_oc_query "MATCH (nodes) RETURN nodes"
    expect(nodes).to eq(make_results(gc))
    edges = db.execute_oc_query "MATCH (edges)-[]->() RETURN edges"

    # Simple create 2
    db.execute_oc_query "CREATE (n1)-[:RELTYPE]->(n2)"
    add_node gc, 3, [], {}
    add_node gc, 4, [], {}
    nodes = db.execute_oc_query "MATCH (nodes) RETURN nodes"
    expect(nodes).to eq(make_results(gc))

    # Create with match
    db.execute_oc_query "CREATE (:X), (:Y)"
    db.execute_oc_query "MATCH (x:X), (y:Y) CREATE (x)-[:R]->(y)"

    # p = db.execute_oc_query "CREATE p = (andres {name:'Andres'})-[:WORKS_AT]->(neo)<-[:WORKS_AT]-(michael {name: 'Michael'}) RETURN p"
  end
end

