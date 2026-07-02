#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include "failure_mode.h"


char* failure_mode_failure_mode_ToString(beater_api_failure_mode__e failure_mode) {
    char *failure_modeArray[] =  { "NULL", "tool_error", "timeout", "guardrail_block", "wrong_output", "retrieval_miss", "other" };
    return failure_modeArray[failure_mode];
}

beater_api_failure_mode__e failure_mode_failure_mode_FromString(char* failure_mode) {
    int stringToReturn = 0;
    char *failure_modeArray[] =  { "NULL", "tool_error", "timeout", "guardrail_block", "wrong_output", "retrieval_miss", "other" };
    size_t sizeofArray = sizeof(failure_modeArray) / sizeof(failure_modeArray[0]);
    while(stringToReturn < sizeofArray) {
        if(strcmp(failure_mode, failure_modeArray[stringToReturn]) == 0) {
            return stringToReturn;
        }
        stringToReturn++;
    }
    return 0;
}

cJSON *failure_mode_convertToJSON(beater_api_failure_mode__e failure_mode) {
    cJSON *item = cJSON_CreateObject();
    if(cJSON_AddStringToObject(item, "failure_mode", failure_mode_failure_mode_ToString(failure_mode)) == NULL) {
        goto fail;
    }
    return item;
fail:
    cJSON_Delete(item);
    return NULL;
}

beater_api_failure_mode__e failure_mode_parseFromJSON(cJSON *failure_modeJSON) {
    if(!cJSON_IsString(failure_modeJSON) || (failure_modeJSON->valuestring == NULL)) {
        return 0;
    }
    return failure_mode_failure_mode_FromString(failure_modeJSON->valuestring);
}
