#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include "api_scope.h"


char* api_scope_api_scope_ToString(beater_api_api_scope__e api_scope) {
    char *api_scopeArray[] =  { "NULL", "trace:write", "trace:read", "dataset:write", "scenario:write", "scenario:read", "eval:run", "pii:unmask", "admin" };
    return api_scopeArray[api_scope];
}

beater_api_api_scope__e api_scope_api_scope_FromString(char* api_scope) {
    int stringToReturn = 0;
    char *api_scopeArray[] =  { "NULL", "trace:write", "trace:read", "dataset:write", "scenario:write", "scenario:read", "eval:run", "pii:unmask", "admin" };
    size_t sizeofArray = sizeof(api_scopeArray) / sizeof(api_scopeArray[0]);
    while(stringToReturn < sizeofArray) {
        if(strcmp(api_scope, api_scopeArray[stringToReturn]) == 0) {
            return stringToReturn;
        }
        stringToReturn++;
    }
    return 0;
}

cJSON *api_scope_convertToJSON(beater_api_api_scope__e api_scope) {
    cJSON *item = cJSON_CreateObject();
    if(cJSON_AddStringToObject(item, "api_scope", api_scope_api_scope_ToString(api_scope)) == NULL) {
        goto fail;
    }
    return item;
fail:
    cJSON_Delete(item);
    return NULL;
}

beater_api_api_scope__e api_scope_parseFromJSON(cJSON *api_scopeJSON) {
    if(!cJSON_IsString(api_scopeJSON) || (api_scopeJSON->valuestring == NULL)) {
        return 0;
    }
    return api_scope_api_scope_FromString(api_scopeJSON->valuestring);
}
