#pragma once

#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

/**
 * The context is needed for most call to the API, it currently is mainly used to handle error reporting.
 * It also manage the memory for any string value returned by the API.
 */
typedef struct gqlite_api_context* gqlite_api_context_t;

/**
 * Connection to the database.
 */
typedef struct gqlite_connection* gqlite_connection_t;

/**
 * Represent a value (integer, double, string, map, array).
 */
typedef struct gqlite_value* gqlite_value_t;

/**
 * Create a new context.
 */
gqlite_api_context_t gqlite_api_context_create();
/**
 * Free context memory.
 */
void gqlite_api_context_destroy(gqlite_api_context_t);
/**
 * Return the error message. The memory is owned by the context,
 * it will be cleared when the context is freed or after the next function call.
 */
const char* gqlite_api_context_get_message(gqlite_api_context_t);
/**
 * Return true if an error has occured in the last call.
 */
bool gqlite_api_context_has_error(gqlite_api_context_t);
/**
 * Clear the error message.
 */
void gqlite_api_context_clear_error(gqlite_api_context_t);

/**
 * Create a connection, using the sqlite backend. Expect as argument an handle to a sqlite connection.
 */
gqlite_connection_t gqlite_connection_create_from_sqlite(gqlite_api_context_t, void* _handle, gqlite_value_t _options);

/**
 * Create a connection, using the sqlite backend. Using the database specified by the filename.
 */
gqlite_connection_t gqlite_connection_create_from_sqlite_file(gqlite_api_context_t, const char* _filename, gqlite_value_t _options);

/**
 * Destroy the connection. Does not delete any connection handle passed as an argument.
 */
void gqlite_connection_destroy(gqlite_api_context_t, gqlite_connection_t);

/**
 * Execute an OpenCypher query on the connection.
 * Return a null value if an error has occured during execution.
 */
gqlite_value_t gqlite_connection_oc_query(gqlite_api_context_t, gqlite_connection_t, const char*, gqlite_value_t);

/**
 * Create a value object to use in a query.
 */
gqlite_value_t gqlite_value_create(gqlite_api_context_t);

/**
 * Destroy a value.
 */
void gqlite_value_destroy(gqlite_api_context_t, gqlite_value_t);

/**
 * Convert value to json. The returned string is only valid until the next API call with the given context.
 */
const char* gqlite_value_to_json(gqlite_api_context_t, gqlite_value_t);

/**
 * Check if the value is valid.
 */
bool gqlite_value_is_valid(gqlite_api_context_t, gqlite_value_t);

#ifdef __cplusplus
}
#endif
