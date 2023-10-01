find_package(Ruby)

if(Ruby_EXECUTABLE)

execute_process(COMMAND ${Ruby_EXECUTABLE} -e "require 'cucumber'" RESULT_VARIABLE CUCUMBER_RESULT_VARIABLE )

if(CUCUMBER_RESULT_VARIABLE EQUAL 0)
set(CUCUMBER_FOUND TRUE)
set(Cucumber_FOUND TRUE)
endif()

endif()
