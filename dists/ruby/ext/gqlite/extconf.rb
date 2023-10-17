require "mkmf"

# Build the gqlite library

# Platforms check
IS_MSWIN = !RbConfig::CONFIG['host_os'].match(/mswin/).nil?
IS_MINGW = !RbConfig::CONFIG['host_os'].match(/mingw/).nil?
IS_DARWIN = !RbConfig::CONFIG['host_os'].match(/darwin/).nil?

# gqlite
if IS_MSWIN
  $CXXFLAGS += " /std:c++20 /EHsc /permissive- /bigobj"
elsif IS_MINGW
  $CXXFLAGS += " -std=c++20 -Wa,-mbig-obj"
  $LDFLAGS += " -lsqlite3 "
else
  $CXXFLAGS += " -std=c++20"
  $LDFLAGS += " -lsqlite3 "
end

create_makefile "gqlite"