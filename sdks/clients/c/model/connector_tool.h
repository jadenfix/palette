/*
 * connector_tool.h
 *
 * A single executable tool within a toolkit, carrying the metadata an agent needs to actually *call* it: the input JSON Schema, tags, and toolkit. This is the raw material for the prompting scaffold in [&#x60;crate::skill&#x60;].
 */

#ifndef _connector_tool_H_
#define _connector_tool_H_

#include <string.h>
#include "../external/cJSON.h"
#include "../include/list.h"
#include "../include/keyValuePair.h"
#include "../include/binary.h"

typedef struct connector_tool_t connector_tool_t;

#include "object.h"



typedef struct connector_tool_t {
    char *description; // string
    object_t *input_schema; //object
    char *name; // string
    int no_auth; //boolean
    char *slug; // string
    list_t *tags; //primitive container
    char *toolkit; // string

    int _library_owned; // Is the library responsible for freeing this object?
} connector_tool_t;

__attribute__((deprecated)) connector_tool_t *connector_tool_create(
    char *description,
    object_t *input_schema,
    char *name,
    int no_auth,
    char *slug,
    list_t *tags,
    char *toolkit
);

void connector_tool_free(connector_tool_t *connector_tool);

connector_tool_t *connector_tool_parseFromJSON(cJSON *connector_toolJSON);

cJSON *connector_tool_convertToJSON(connector_tool_t *connector_tool);

#endif /* _connector_tool_H_ */

