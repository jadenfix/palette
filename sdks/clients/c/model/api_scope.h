/*
 * api_scope.h
 *
 * 
 */

#ifndef _api_scope_H_
#define _api_scope_H_

#include <string.h>
#include "../external/cJSON.h"
#include "../include/list.h"
#include "../include/keyValuePair.h"
#include "../include/binary.h"

typedef struct api_scope_t api_scope_t;


// Enum  for api_scope

typedef enum { beater_api_api_scope__NULL = 0, beater_api_api_scope__trace:write, beater_api_api_scope__trace:read, beater_api_api_scope__dataset:write, beater_api_api_scope__scenario:write, beater_api_api_scope__scenario:read, beater_api_api_scope__eval:run, beater_api_api_scope__pii:unmask, beater_api_api_scope__admin } beater_api_api_scope__e;

char* api_scope_api_scope_ToString(beater_api_api_scope__e api_scope);

beater_api_api_scope__e api_scope_api_scope_FromString(char* api_scope);

cJSON *api_scope_convertToJSON(beater_api_api_scope__e api_scope);

beater_api_api_scope__e api_scope_parseFromJSON(cJSON *api_scopeJSON);

#endif /* _api_scope_H_ */

