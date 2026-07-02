/*
 * api_key_created_response.h
 *
 * 
 */

#ifndef _api_key_created_response_H_
#define _api_key_created_response_H_

#include <string.h>
#include "../external/cJSON.h"
#include "../include/list.h"
#include "../include/keyValuePair.h"
#include "../include/binary.h"

typedef struct api_key_created_response_t api_key_created_response_t;

#include "api_scope.h"

// Enum  for api_key_created_response

typedef enum  { beater_api_api_key_created_response__NULL = 0, beater_api_api_key_created_response__trace_write, beater_api_api_key_created_response__trace_read, beater_api_api_key_created_response__dataset_write, beater_api_api_key_created_response__scenario_write, beater_api_api_key_created_response__scenario_read, beater_api_api_key_created_response__eval_run, beater_api_api_key_created_response__pii_unmask, beater_api_api_key_created_response__admin } beater_api_api_key_created_response__e;

char* api_key_created_response_scopes_ToString(beater_api_api_key_created_response__e scopes);

beater_api_api_key_created_response__e api_key_created_response_scopes_FromString(char* scopes);



typedef struct api_key_created_response_t {
    int active; //boolean
    char *api_key_id; // string
    char *created_at; //date time
    char *environment_id; // string
    char *project_id; // string
    list_t *scopes; //nonprimitive container
    char *secret; // string
    char *tenant_id; // string

    int _library_owned; // Is the library responsible for freeing this object?
} api_key_created_response_t;

__attribute__((deprecated)) api_key_created_response_t *api_key_created_response_create(
    int active,
    char *api_key_id,
    char *created_at,
    char *environment_id,
    char *project_id,
    list_t *scopes,
    char *secret,
    char *tenant_id
);

void api_key_created_response_free(api_key_created_response_t *api_key_created_response);

api_key_created_response_t *api_key_created_response_parseFromJSON(cJSON *api_key_created_responseJSON);

cJSON *api_key_created_response_convertToJSON(api_key_created_response_t *api_key_created_response);

#endif /* _api_key_created_response_H_ */

