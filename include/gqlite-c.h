#pragma once

#ifdef __cplusplus
extern "C" {
#endif

typedef struct gqlite_api_context* gqlite_api_context_t;
typedef struct gqlite_database* gqlite_database_t;
typedef struct gqlite_backend* gqlite_backend_t;
typedef struct gqlite_value* gqlite_value_t;

gqlite_api_context_t gqlite_api_context_create();
void gqlite_api_context_destroy(gqlite_api_context_t);
const char* gqlite_api_context_get_message(gqlite_api_context_t);
bool gqlite_api_context_has_error(gqlite_api_context_t);
void gqlite_api_context_clear_error(gqlite_api_context_t);

/**
 * Create a database, using the sqlite backend. Expect as argument an handle to a sqlite database.
 */
gqlite_database_t gqlite_database_create_from_sqlite(gqlite_api_context_t, void*);

gqlite_database_t gqlite_database_create_from_sqlite_file(gqlite_api_context_t, const char*);

/**
 * Destroy the database. Does not delete any database handle passed as an argument.
 */
void gqlite_database_destroy(gqlite_api_context_t, gqlite_database_t);

/**
 * Execute an OpenCypher query on the database.
 */
gqlite_value_t gqlite_database_oc_query(gqlite_api_context_t, gqlite_database_t, const char*, gqlite_value_t);

/**
 * Create a value object to use in a query.
 */
gqlite_value_t gqlite_value_create(gqlite_api_context_t);
void gqlite_value_destroy(gqlite_api_context_t, gqlite_value_t);
const char* gqlite_value_to_json(gqlite_api_context_t, gqlite_value_t);
bool gqlite_value_is_valid(gqlite_api_context_t, gqlite_value_t);

#ifdef __cplusplus
}
#endif
