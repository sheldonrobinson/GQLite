require "json"

@[Link("gqlitedb")]
lib LibGQLite
  fun gqlite_api_context_create : Void*
  fun gqlite_api_context_destroy(Void*) : Void*
  fun gqlite_api_context_clear_error(Void*) : Void*
  fun gqlite_api_context_has_error(Void*) : Bool
  fun gqlite_api_context_get_message(Void*) : UInt8*
  fun gqlite_connection_create_from_file(Void*, UInt8*, Void*) : Void*
  fun gqlite_connection_destroy(Void*, Void*) : Void*
  fun gqlite_connection_query(Void*, Void*, UInt8*, Void*) : Void*
  fun gqlite_value_create(Void*) : Void*
  fun gqlite_value_destroy(Void*, Void*) : Void*
  fun gqlite_value_to_json(Void*, Void*) : UInt8*
  fun gqlite_value_is_valid(Void*, Void*) : Bool
end

module GQLite
  class Error < Exception
  end

  ApiContext = LibGQLite.gqlite_api_context_create

  def GQLite.handle_api_context(r)
    if LibGQLite.gqlite_api_context_has_error(ApiContext)
      err = LibGQLite.gqlite_api_context_get_message ApiContext
      err_msg = String.new(err)
      LibGQLite.gqlite_api_context_clear_error ApiContext
      raise Error.new err_msg
    end
    return r
  end

  class Connection
    @dbhandle : Void*

    def initialize(filename = nil)
      if filename != nil
        @dbhandle = GQLite.handle_api_context(LibGQLite.gqlite_connection_create_from_file ApiContext, filename, Pointer(Void).null)
      else
        raise Error.new "No connection backend was selected."
      end
      # ObjectSpace.define_finalizer @dbhandle, proc {|id|
      # GQLite.call_function ->LibGQLite.gqlite_connection_destroy, @dbhandle
      # }
    end

    def execute_oc_query(query, bindings = nil)
      ret = GQLite.handle_api_context(LibGQLite.gqlite_connection_query ApiContext, @dbhandle, query, nil)
      if GQLite.handle_api_context(LibGQLite.gqlite_value_is_valid ApiContext, ret)
        val = GQLite.handle_api_context(LibGQLite.gqlite_value_to_json ApiContext, ret)
        val = JSON.parse(String.new(val))
      else
        val = nil
      end
      GQLite.handle_api_context(LibGQLite.gqlite_value_destroy ApiContext, ret)
      return val
    end
  end
end
