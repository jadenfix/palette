/*
 * connector_skills_response.h
 *
 * Generated prompting scaffold (\&quot;skills.md\&quot;) for a toolkit&#39;s tools.
 */

#ifndef _connector_skills_response_H_
#define _connector_skills_response_H_

#include <string.h>
#include "../external/cJSON.h"
#include "../include/list.h"
#include "../include/keyValuePair.h"
#include "../include/binary.h"

typedef struct connector_skills_response_t connector_skills_response_t;




typedef struct connector_skills_response_t {
    char *skills; // string
    char *toolkit; // string

    int _library_owned; // Is the library responsible for freeing this object?
} connector_skills_response_t;

__attribute__((deprecated)) connector_skills_response_t *connector_skills_response_create(
    char *skills,
    char *toolkit
);

void connector_skills_response_free(connector_skills_response_t *connector_skills_response);

connector_skills_response_t *connector_skills_response_parseFromJSON(cJSON *connector_skills_responseJSON);

cJSON *connector_skills_response_convertToJSON(connector_skills_response_t *connector_skills_response);

#endif /* _connector_skills_response_H_ */

