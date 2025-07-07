require 'ffi'
require 'objspace'
require 'json'

module GQLite
  class Error < StandardError
  end
  module CApi
    extend FFI::Library

    # Attempt to find the gqlite build that is shipped with gem
    def CApi.get_lib_name()
      path = "#{File.dirname __FILE__}/#{FFI::Platform::LIBPREFIX}gqlite.#{FFI::Platform::LIBSUFFIX}"
      return path if File.exist?(path)
      path = "#{File.dirname __FILE__}/gqlite.#{FFI::Platform::LIBSUFFIX}"
      return path if File.exist?(path)
      return "gqlite"
    end

    ffi_lib CApi.get_lib_name()
    attach_function :gqlite_api_context_create, [], :pointer
    attach_function :gqlite_api_context_destroy, [:pointer], :void
    attach_function :gqlite_api_context_clear_error, [:pointer], :void
    attach_function :gqlite_api_context_has_error, [:pointer], :bool
    attach_function :gqlite_api_context_get_message, [:pointer], :string
    attach_function :gqlite_connection_create_from_file, [:pointer, :string, :pointer], :pointer
    attach_function :gqlite_connection_destroy, [:pointer, :pointer], :void
    attach_function :gqlite_connection_query, [:pointer, :pointer, :string, :pointer], :pointer
    attach_function :gqlite_value_create, [:pointer], :pointer
    attach_function :gqlite_value_destroy, [:pointer, :pointer], :void
    attach_function :gqlite_value_to_json, [:pointer, :pointer], :string
    attach_function :gqlite_value_from_json, [:pointer, :string], :pointer
    attach_function :gqlite_value_is_valid, [:pointer, :pointer], :bool
    ApiContext = CApi.gqlite_api_context_create()
    def CApi.call_function(fname, *args)
      r = CApi.send fname, ApiContext, *args
      if CApi.gqlite_api_context_has_error(ApiContext)
        err = CApi.gqlite_api_context_get_message ApiContext
        CApi.gqlite_api_context_clear_error ApiContext
        raise Error.new err
      end
      return r
    end
  end
  class Connection
    attr_reader :dbhandle
    def initialize(filename: nil, backend: nil)
      options = {}
      unless backend.nil?
        options["backend"] = backend
      end
      options = CApi.call_function :gqlite_value_from_json, options.to_json
      if filename != nil
        @dbhandle = CApi.call_function :gqlite_connection_create_from_file, filename, options
        CApi.call_function :gqlite_value_destroy, options
      else
        CApi.call_function :gqlite_value_destroy, options
        raise Error.new "No connection backend was selected."
      end
      ObjectSpace.define_finalizer @dbhandle, proc {|id|
        CApi.call_function :gqlite_connection_destroy, @dbhandle
      }
    end
    def execute_oc_query(query, bindings: nil)
      b = nil
      if bindings
        b = CApi.call_function :gqlite_value_from_json, bindings.to_json 
      end
      ret = CApi.call_function :gqlite_connection_query, @dbhandle, query, b
      if b
        CApi.call_function :gqlite_value_destroy, b
      end
      if CApi.call_function(:gqlite_value_is_valid, ret)
        val = JSON.parse(CApi.call_function :gqlite_value_to_json, ret)
      else
        val = nil
      end
      CApi.call_function :gqlite_value_destroy, ret
      return val
    end
  end
end
