/*
 * invoke_connector_request.h
 *
 * 
 */

#ifndef _invoke_connector_request_H_
#define _invoke_connector_request_H_

#include <string.h>
#include "../external/cJSON.h"
#include "../include/list.h"
#include "../include/keyValuePair.h"
#include "../include/binary.h"

typedef struct invoke_connector_request_t invoke_connector_request_t;

#include "object.h"



typedef struct invoke_connector_request_t {
    object_t *arguments; //object
    char *tool; // string

    int _library_owned; // Is the library responsible for freeing this object?
} invoke_connector_request_t;

__attribute__((deprecated)) invoke_connector_request_t *invoke_connector_request_create(
    object_t *arguments,
    char *tool
);

void invoke_connector_request_free(invoke_connector_request_t *invoke_connector_request);

invoke_connector_request_t *invoke_connector_request_parseFromJSON(cJSON *invoke_connector_requestJSON);

cJSON *invoke_connector_request_convertToJSON(invoke_connector_request_t *invoke_connector_request);

#endif /* _invoke_connector_request_H_ */

