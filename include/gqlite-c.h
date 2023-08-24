#pragma once

#ifdef __cplusplus
extern "C" {
#endif

typedef struct gqlite_api_error* gqlite_api_error_t;
typedef struct gqlite_database* gqlite_database_t;
typedef struct gqlite_backend* gqlite_backend_t;
typedef struct gqlite_result* gqlite_result_t;
typedef struct gqlite_bindings* gqlite_bindings_t;

gqlite_api_error_t gqlite_api_error_create();
void gqlite_api_error_destroy(gqlite_api_error_t);
const char* gqlite_api_error_get_message(gqlite_api_error_t);
bool gqlite_api_error_has_error(gqlite_api_error_t);

/**
 * Create a database, using the sqlite backend. Expect as argument an handle to a sqlite database.
 */
gqlite_database_t gqlite_database_create_from_sqlite(gqlite_api_error_t, void*);

gqlite_database_t gqlite_database_create_from_sqlite_file(gqlite_api_error_t, const char*);

/**
 * Destroy the database. Does not delete any database handle passed as an argument.
 */
void gqlite_database_destroy(gqlite_api_error_t, gqlite_database_t);

/**
 * Execute an OpenCypher query on the database.
 */
gqlite_result_t gqlite_database_oc_query(gqlite_api_error_t, gqlite_database_t, const char*, gqlite_bindings_t);

void gqlite_result_destroy(gqlite_api_error_t, gqlite_result_t);
const char* gqlite_result_error(gqlite_api_error_t, gqlite_result_t);

enum { GQLITE_RESULT_SUCCESS, GQLITE_RESULT_ERROR };

int gqlite_result_status(gqlite_api_error_t, gqlite_result_t);

/**
 * Create a bindings object to use in a query.
 */
gqlite_bindings_t gqlite_bindings_create(gqlite_api_error_t);
void gqlite_bindings_destroy(gqlite_api_error_t, gqlite_bindings_t);

#ifdef __cplusplus
}
#endif
