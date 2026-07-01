/*
 * connection_link.h
 *
 * One-time login link returned when initiating a managed-OAuth connection.
 */

#ifndef _connection_link_H_
#define _connection_link_H_

#include <string.h>
#include "../external/cJSON.h"
#include "../include/list.h"
#include "../include/keyValuePair.h"
#include "../include/binary.h"

typedef struct connection_link_t connection_link_t;




typedef struct connection_link_t {
    char *connected_account_id; // string
    char *expires_at; // string
    char *redirect_url; // string

    int _library_owned; // Is the library responsible for freeing this object?
} connection_link_t;

__attribute__((deprecated)) connection_link_t *connection_link_create(
    char *connected_account_id,
    char *expires_at,
    char *redirect_url
);

void connection_link_free(connection_link_t *connection_link);

connection_link_t *connection_link_parseFromJSON(cJSON *connection_linkJSON);

cJSON *connection_link_convertToJSON(connection_link_t *connection_link);

#endif /* _connection_link_H_ */

