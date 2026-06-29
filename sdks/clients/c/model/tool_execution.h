/*
 * tool_execution.h
 *
 * Result of executing a tool — Composio&#39;s &#x60;{successful, data, error}&#x60; envelope.
 */

#ifndef _tool_execution_H_
#define _tool_execution_H_

#include <string.h>
#include "../external/cJSON.h"
#include "../include/list.h"
#include "../include/keyValuePair.h"
#include "../include/binary.h"

typedef struct tool_execution_t tool_execution_t;

#include "object.h"



typedef struct tool_execution_t {
    object_t *data; //object
    char *error; // string
    char *log_id; // string
    int successful; //boolean

    int _library_owned; // Is the library responsible for freeing this object?
} tool_execution_t;

__attribute__((deprecated)) tool_execution_t *tool_execution_create(
    object_t *data,
    char *error,
    char *log_id,
    int successful
);

void tool_execution_free(tool_execution_t *tool_execution);

tool_execution_t *tool_execution_parseFromJSON(cJSON *tool_executionJSON);

cJSON *tool_execution_convertToJSON(tool_execution_t *tool_execution);

#endif /* _tool_execution_H_ */

