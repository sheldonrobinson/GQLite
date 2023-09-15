require "spec"
require "gqlite"

describe GQLite::Connection do
  it "can create db and query them" do
    file = File.tempfile("testdb")
    c = GQLite::Connection.new(file.path)
    c.execute_oc_query("CREATE (n)")
    m = c.execute_oc_query("MATCH (n) RETURN n")
    m.should eq [["n"], [{"properties" => {} of String => Nil, "id" => 1, "labels" => [] of String, "type" => "node"}]]
    expect_raises(GQLite::Error, "1:8:Expected token ) got end of file") do
      c.execute_oc_query("MATCH (n")
    end
  end
end
