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
    db.execute_oc_query "CREATE (n1:Person)"
    db.execute_oc_query "CREATE (n1:Person), (n2:Film)"
    db.execute_oc_query "CREATE (n1 {name: 'Andres', title: 'Developer'})"
    db.execute_oc_query "CREATE (n1:Person {name: 'Andres', title: 'Developer'})"
  end
end
