/*
 * error_response.h
 *
 * Error envelope returned by every fallible endpoint.
 */

#ifndef _error_response_H_
#define _error_response_H_

#include <string.h>
#include "../external/cJSON.h"
#include "../include/list.h"
#include "../include/keyValuePair.h"
#include "../include/binary.h"

typedef struct error_response_t error_response_t;




typedef struct error_response_t {
    char *error; // string
    char *message; // string
    int status; //numeric

    int _library_owned; // Is the library responsible for freeing this object?
} error_response_t;

__attribute__((deprecated)) error_response_t *error_response_create(
    char *error,
    char *message,
    int status
);

void error_response_free(error_response_t *error_response);

error_response_t *error_response_parseFromJSON(cJSON *error_responseJSON);

cJSON *error_response_convertToJSON(error_response_t *error_response);

#endif /* _error_response_H_ */

