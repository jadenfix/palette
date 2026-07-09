#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include "palette_connect_status.h"


char* palette_connect_status_palette_connect_status_ToString(beater_api_palette_connect_status__e palette_connect_status) {
    char *palette_connect_statusArray[] =  { "NULL", "connected", "waiting_for_trace", "waiting_for_eval", "misconfigured" };
    return palette_connect_statusArray[palette_connect_status];
}

beater_api_palette_connect_status__e palette_connect_status_palette_connect_status_FromString(char* palette_connect_status) {
    int stringToReturn = 0;
    char *palette_connect_statusArray[] =  { "NULL", "connected", "waiting_for_trace", "waiting_for_eval", "misconfigured" };
    size_t sizeofArray = sizeof(palette_connect_statusArray) / sizeof(palette_connect_statusArray[0]);
    while(stringToReturn < sizeofArray) {
        if(strcmp(palette_connect_status, palette_connect_statusArray[stringToReturn]) == 0) {
            return stringToReturn;
        }
        stringToReturn++;
    }
    return 0;
}

cJSON *palette_connect_status_convertToJSON(beater_api_palette_connect_status__e palette_connect_status) {
    cJSON *item = cJSON_CreateObject();
    if(cJSON_AddStringToObject(item, "palette_connect_status", palette_connect_status_palette_connect_status_ToString(palette_connect_status)) == NULL) {
        goto fail;
    }
    return item;
fail:
    cJSON_Delete(item);
    return NULL;
}

beater_api_palette_connect_status__e palette_connect_status_parseFromJSON(cJSON *palette_connect_statusJSON) {
    if(!cJSON_IsString(palette_connect_statusJSON) || (palette_connect_statusJSON->valuestring == NULL)) {
        return 0;
    }
    return palette_connect_status_palette_connect_status_FromString(palette_connect_statusJSON->valuestring);
}
