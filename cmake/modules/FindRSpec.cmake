find_package(Ruby)

if(RUBY_FOUND)

execute_process(COMMAND ${RUBY_EXECUTABLE} -e "require 'rspec'" RESULT_VARIABLE RSPEC_RESULT_VARIABLE )

if(RSPEC_RESULT_VARIABLE EQUAL 0)
set(RSPEC_FOUND TRUE)
set(RSpec_FOUND TRUE)

find_program(RSPEC_EXECUTABLE rspec)

endif()

endif()
