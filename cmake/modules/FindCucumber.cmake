find_package(Ruby)

if(RUBY_FOUND)

execute_process(COMMAND ${RUBY_EXECUTABLE} -e "require 'cucumber'" RESULT_VARIABLE CUCUMBER_RESULT_VARIABLE )

if(CUCUMBER_RESULT_VARIABLE EQUAL 0)
set(CUCUMBER_FOUND TRUE)
set(Cucumber_FOUND TRUE)
endif()

endif()
