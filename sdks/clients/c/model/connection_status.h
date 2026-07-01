/*
 * connection_status.h
 *
 * Connection status of one app for one entity.
 */

#ifndef _connection_status_H_
#define _connection_status_H_

#include <string.h>
#include "../external/cJSON.h"
#include "../include/list.h"
#include "../include/keyValuePair.h"
#include "../include/binary.h"

typedef struct connection_status_t connection_status_t;




typedef struct connection_status_t {
    int connected; //boolean
    char *connected_account_id; // string
    char *status; // string
    char *toolkit; // string

    int _library_owned; // Is the library responsible for freeing this object?
} connection_status_t;

__attribute__((deprecated)) connection_status_t *connection_status_create(
    int connected,
    char *connected_account_id,
    char *status,
    char *toolkit
);

void connection_status_free(connection_status_t *connection_status);

connection_status_t *connection_status_parseFromJSON(cJSON *connection_statusJSON);

cJSON *connection_status_convertToJSON(connection_status_t *connection_status);

#endif /* _connection_status_H_ */

