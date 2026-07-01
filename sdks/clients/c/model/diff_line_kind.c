#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include "diff_line_kind.h"


char* diff_line_kind_diff_line_kind_ToString(beater_api_diff_line_kind__e diff_line_kind) {
    char *diff_line_kindArray[] =  { "NULL", "unchanged", "added", "removed" };
    return diff_line_kindArray[diff_line_kind];
}

beater_api_diff_line_kind__e diff_line_kind_diff_line_kind_FromString(char* diff_line_kind) {
    int stringToReturn = 0;
    char *diff_line_kindArray[] =  { "NULL", "unchanged", "added", "removed" };
    size_t sizeofArray = sizeof(diff_line_kindArray) / sizeof(diff_line_kindArray[0]);
    while(stringToReturn < sizeofArray) {
        if(strcmp(diff_line_kind, diff_line_kindArray[stringToReturn]) == 0) {
            return stringToReturn;
        }
        stringToReturn++;
    }
    return 0;
}

cJSON *diff_line_kind_convertToJSON(beater_api_diff_line_kind__e diff_line_kind) {
    cJSON *item = cJSON_CreateObject();
    if(cJSON_AddStringToObject(item, "diff_line_kind", diff_line_kind_diff_line_kind_ToString(diff_line_kind)) == NULL) {
        goto fail;
    }
    return item;
fail:
    cJSON_Delete(item);
    return NULL;
}

beater_api_diff_line_kind__e diff_line_kind_parseFromJSON(cJSON *diff_line_kindJSON) {
    if(!cJSON_IsString(diff_line_kindJSON) || (diff_line_kindJSON->valuestring == NULL)) {
        return 0;
    }
    return diff_line_kind_diff_line_kind_FromString(diff_line_kindJSON->valuestring);
}
