/*
 * connect_connector_request.h
 *
 * 
 */

#ifndef _connect_connector_request_H_
#define _connect_connector_request_H_

#include <string.h>
#include "../external/cJSON.h"
#include "../include/list.h"
#include "../include/keyValuePair.h"
#include "../include/binary.h"

typedef struct connect_connector_request_t connect_connector_request_t;




typedef struct connect_connector_request_t {
    char *toolkit; // string

    int _library_owned; // Is the library responsible for freeing this object?
} connect_connector_request_t;

__attribute__((deprecated)) connect_connector_request_t *connect_connector_request_create(
    char *toolkit
);

void connect_connector_request_free(connect_connector_request_t *connect_connector_request);

connect_connector_request_t *connect_connector_request_parseFromJSON(cJSON *connect_connector_requestJSON);

cJSON *connect_connector_request_convertToJSON(connect_connector_request_t *connect_connector_request);

#endif /* _connect_connector_request_H_ */

