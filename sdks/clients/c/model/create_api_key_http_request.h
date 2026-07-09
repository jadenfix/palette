/*
 * create_api_key_http_request.h
 *
 * 
 */

#ifndef _create_api_key_http_request_H_
#define _create_api_key_http_request_H_

#include <string.h>
#include "../external/cJSON.h"
#include "../include/list.h"
#include "../include/keyValuePair.h"
#include "../include/binary.h"

typedef struct create_api_key_http_request_t create_api_key_http_request_t;

#include "api_scope.h"

// Enum  for create_api_key_http_request

typedef enum  { beater_api_create_api_key_http_request__NULL = 0, beater_api_create_api_key_http_request__trace:write, beater_api_create_api_key_http_request__trace:read, beater_api_create_api_key_http_request__dataset:write, beater_api_create_api_key_http_request__scenario:write, beater_api_create_api_key_http_request__scenario:read, beater_api_create_api_key_http_request__eval:run, beater_api_create_api_key_http_request__pii:unmask, beater_api_create_api_key_http_request__admin } beater_api_create_api_key_http_request__e;

char* create_api_key_http_request_scopes_ToString(beater_api_create_api_key_http_request__e scopes);

beater_api_create_api_key_http_request__e create_api_key_http_request_scopes_FromString(char* scopes);



typedef struct create_api_key_http_request_t {
    list_t *scopes; //nonprimitive container

    int _library_owned; // Is the library responsible for freeing this object?
} create_api_key_http_request_t;

__attribute__((deprecated)) create_api_key_http_request_t *create_api_key_http_request_create(
    list_t *scopes
);

void create_api_key_http_request_free(create_api_key_http_request_t *create_api_key_http_request);

create_api_key_http_request_t *create_api_key_http_request_parseFromJSON(cJSON *create_api_key_http_requestJSON);

cJSON *create_api_key_http_request_convertToJSON(create_api_key_http_request_t *create_api_key_http_request);

#endif /* _create_api_key_http_request_H_ */

