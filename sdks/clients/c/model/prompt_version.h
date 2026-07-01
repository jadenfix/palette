/*
 * prompt_version.h
 *
 * 
 */

#ifndef _prompt_version_H_
#define _prompt_version_H_

#include <string.h>
#include "../external/cJSON.h"
#include "../include/list.h"
#include "../include/keyValuePair.h"
#include "../include/binary.h"

typedef struct prompt_version_t prompt_version_t;

#include "prompt_template.h"
#include "prompt_version_metadata.h"



typedef struct prompt_version_t {
    struct prompt_version_metadata_t *metadata; //model
    char *project_id; // string
    char *prompt_id; // string
    struct prompt_template_t *_template; //model
    char *tenant_id; // string
    char *version_id; // string
    int version_number; //numeric

    int _library_owned; // Is the library responsible for freeing this object?
} prompt_version_t;

__attribute__((deprecated)) prompt_version_t *prompt_version_create(
    prompt_version_metadata_t *metadata,
    char *project_id,
    char *prompt_id,
    prompt_template_t *_template,
    char *tenant_id,
    char *version_id,
    int version_number
);

void prompt_version_free(prompt_version_t *prompt_version);

prompt_version_t *prompt_version_parseFromJSON(cJSON *prompt_versionJSON);

cJSON *prompt_version_convertToJSON(prompt_version_t *prompt_version);

#endif /* _prompt_version_H_ */

