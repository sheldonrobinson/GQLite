#include <gqlite.h>

#include <iostream>

int main(int _argc, char** _argv)
{
  if(_argc != 3)
  {
    printf("gqlite_cpp_example [database] [query]");
    exit(-1);
  }

  try
  {
    // Create a database on the file given in argv[1]
    gqlite::connection connection = gqlite::connection::create_from_file(_argv[1]);

    // Execute the query from argv[2]
    gqlite::value value = connection.execute_oc_query(_argv[2]);

    // Convert the result to json and print it
    std::cout << "Result: " << value.to_json() << std::endl;
    return 0;
  } catch(const gqlite::exception& _ex)
  {
    // Errors with the C++ API are reported with exceptions. The exceptions should be caught.
    std::cerr << "An error has occured: " << _ex.what() << std::endl;
    return -1;
  }
}
