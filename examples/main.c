#include <gqlite-c.h>

#include <stdio.h>
#include <stdlib.h>

void check_error(gqlite_api_context_t context)
{
  // Always check if an error has occured after an API call.
  if(gqlite_api_context_has_error(context))
  {
    // Print the error message
    printf("An error has occured: %s\n", gqlite_api_context_get_message(context));
    // Clear the error
    gqlite_api_context_clear_error(context);
    exit(-1);
  }
}

int main(int _argc, char** _argv)
{
  if(_argc != 3)
  {
    printf("gqlite_c_example [database] [query]");
    exit(-1);
  }
  // Create a context
  gqlite_api_context_t context = gqlite_api_context_create();

  // Create a database on the file given in argv[1]
  gqlite_connection_t connection = gqlite_connection_create_from_sqlite_file(context, _argv[1], NULL);
  check_error(context);

  // Execute the query from argv[2]
  gqlite_value_t value = gqlite_connection_oc_query(context, connection, _argv[2], NULL);
  check_error(context);

  // Convert the result to json and print it
  const char* string = gqlite_value_to_json(context, value);
  check_error(context);
  printf("Result: %s\n", string);
  gqlite_value_destroy(context, value);
}
