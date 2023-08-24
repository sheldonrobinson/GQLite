require 'rspec'
require 'gqlite'

RSpec.describe "database" do
  it "can be created" do
    db = Gqlite::Database.new(sqlite_filename: "testdb")
  end
  it "can be queried with oc to create nodes" do
    db = Gqlite::Database.new(sqlite_filename: "testdb")
    db.execute_oc_query "CREATE (n1)"
    db.execute_oc_query "CREATE (n1, n2)"
  end
end
