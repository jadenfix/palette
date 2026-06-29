#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include "audit_action.h"


char* audit_action_audit_action_ToString(beater_api_audit_action__e audit_action) {
    char *audit_actionArray[] =  { "NULL", "pii_unmask", "api_key_create", "api_key_revoke", "provider_secret_create", "provider_secret_revoke" };
    return audit_actionArray[audit_action];
}

beater_api_audit_action__e audit_action_audit_action_FromString(char* audit_action) {
    int stringToReturn = 0;
    char *audit_actionArray[] =  { "NULL", "pii_unmask", "api_key_create", "api_key_revoke", "provider_secret_create", "provider_secret_revoke" };
    size_t sizeofArray = sizeof(audit_actionArray) / sizeof(audit_actionArray[0]);
    while(stringToReturn < sizeofArray) {
        if(strcmp(audit_action, audit_actionArray[stringToReturn]) == 0) {
            return stringToReturn;
        }
        stringToReturn++;
    }
    return 0;
}

cJSON *audit_action_convertToJSON(beater_api_audit_action__e audit_action) {
    cJSON *item = cJSON_CreateObject();
    if(cJSON_AddStringToObject(item, "audit_action", audit_action_audit_action_ToString(audit_action)) == NULL) {
        goto fail;
    }
    return item;
fail:
    cJSON_Delete(item);
    return NULL;
}

beater_api_audit_action__e audit_action_parseFromJSON(cJSON *audit_actionJSON) {
    if(!cJSON_IsString(audit_actionJSON) || (audit_actionJSON->valuestring == NULL)) {
        return 0;
    }
    return audit_action_audit_action_FromString(audit_actionJSON->valuestring);
}
