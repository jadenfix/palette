/*
 * toolkit.h
 *
 * A connectable third-party app (Composio \&quot;toolkit\&quot;), flattened from the v3 &#x60;GET /toolkits&#x60; shape into the fields Beater exposes.
 */

#ifndef _toolkit_H_
#define _toolkit_H_

#include <string.h>
#include "../external/cJSON.h"
#include "../include/list.h"
#include "../include/keyValuePair.h"
#include "../include/binary.h"

typedef struct toolkit_t toolkit_t;




typedef struct toolkit_t {
    list_t *auth_schemes; //primitive container
    char *description; // string
    char *name; // string
    int no_auth; //boolean
    char *slug; // string
    int tools_count; //numeric

    int _library_owned; // Is the library responsible for freeing this object?
} toolkit_t;

__attribute__((deprecated)) toolkit_t *toolkit_create(
    list_t *auth_schemes,
    char *description,
    char *name,
    int no_auth,
    char *slug,
    int tools_count
);

void toolkit_free(toolkit_t *toolkit);

toolkit_t *toolkit_parseFromJSON(cJSON *toolkitJSON);

cJSON *toolkit_convertToJSON(toolkit_t *toolkit);

#endif /* _toolkit_H_ */

