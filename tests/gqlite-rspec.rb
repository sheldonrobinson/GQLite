require 'rspec'
require 'gqlite'

RSpec.describe "database" do
  it "can be created" do
    db = Gqlite::Database.new(sqlite_filename: "testdb")
  end
  it "can be queried with oc to create nodes" do
    db = Gqlite::Database.new(sqlite_filename: "testdb")
    db.execute_oc_query "CREATE (n1)"
    db.execute_oc_query "CREATE (n1), (n2)"
    db.execute_oc_query "CREATE (n1:Person)"
    db.execute_oc_query "CREATE (n1:Person), (n2:Film)"
    db.execute_oc_query "CREATE (n1 {name: 'Andres', title: 'Developer'})"
    db.execute_oc_query "CREATE (n1 {name: 'Andres', title: 'Developer'})"
  end
end
