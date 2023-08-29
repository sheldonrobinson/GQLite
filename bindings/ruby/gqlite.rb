require 'ffi'
require 'objspace'

module Gqlite
  class Error < StandardError
  end
  module CApi
    extend FFI::Library
    ffi_lib 'gqlite'
    attach_function :gqlite_api_context_create, [], :pointer
    attach_function :gqlite_api_context_destroy, [:pointer], :void
    attach_function :gqlite_api_context_clear_error, [:pointer], :void
    attach_function :gqlite_api_context_has_error, [:pointer], :bool
    attach_function :gqlite_api_context_get_message, [:pointer], :string
    attach_function :gqlite_database_create_from_sqlite_file, [:pointer, :string], :pointer
    attach_function :gqlite_database_destroy, [:pointer, :pointer], :void
    attach_function :gqlite_database_oc_query, [:pointer, :pointer, :string, :pointer], :pointer
    attach_function :gqlite_value_create, [:pointer], :pointer
    attach_function :gqlite_value_destroy, [:pointer, :pointer], :void
    ApiError = CApi.gqlite_api_context_create()
    def CApi.call_function(fname, *args)
      r = CApi.send fname, ApiError, *args
      if CApi.gqlite_api_context_has_error(ApiError)
        err = CApi.gqlite_api_context_get_message ApiError
        CApi.gqlite_api_context_clear_error ApiError
        raise Error.new err
      end
      return r
    end
  end
  class Database
    def initialize(sqlite_filename: nil)
      if sqlite_filename != nil
        @dbhandle = CApi.call_function :gqlite_database_create_from_sqlite_file, sqlite_filename
      else
        raise Error.new "No database backend was selected."
      end
      ObjectSpace.define_finalizer @dbhandle, proc {|id|
        CApi.call_function :gqlite_database_destroy, @dbhandle
      }
    end
    def execute_oc_query(query, bindings: nil)
      return CApi.call_function :gqlite_database_oc_query, @dbhandle, query, nil
    end
  end
end
