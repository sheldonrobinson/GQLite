require 'rspec'
require 'gqlite'
require 'tempfile'

RSpec.describe "database" do
  it "can be created" do
    file = Tempfile.new('testdb')
    db = Gqlite::Database.new(sqlite_filename: file.path)
  end
  it "can be queried with oc to create nodes" do
    file = Tempfile.new('testdb')
    db = Gqlite::Database.new(sqlite_filename: file.path)
    db.execute_oc_query "CREATE (n1)"
    db.execute_oc_query "CREATE (n1), (n2)"
    n1 = db.execute_oc_query "CREATE (n1:Person) RETURN n1"
    expect(n1).to eq([{"type" => "node", "id" => 4, "labels" => ["Person"], "properties" => {}}])
    db.execute_oc_query "CREATE (n1:Person), (n2:Film)"
    db.execute_oc_query "CREATE (n1 {name: 'Andres', title: 'Developer'})"
    n1 = db.execute_oc_query "CREATE (n1:Person {name: 'Andres', title: 'Developer'}) RETURN n1"
    expect(n1).to eq([{"type" => "node", "id" => 8, "labels" => ["Person"], "properties" => {"name" => 'Andres', "title" => 'Developer'}}]) 
  end
  it "can be queried with oc to create nodes and edges" do
    file = Tempfile.new('testdb')
    db = Gqlite::Database.new(sqlite_filename: file.path)
    db.execute_oc_query "CREATE (n1), (n2) CREATE (n1)-[:RELTYPE]->(n2)"
    p = db.execute_oc_query "CREATE p = (andres {name:'Andres'})-[:WORKS_AT]->(neo)<-[:WORKS_AT]-(michael {name: 'Michael'}) RETURN p"
  end
end

