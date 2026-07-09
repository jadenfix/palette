/*
 * palette_connect_status.h
 *
 * 
 */

#ifndef _palette_connect_status_H_
#define _palette_connect_status_H_

#include <string.h>
#include "../external/cJSON.h"
#include "../include/list.h"
#include "../include/keyValuePair.h"
#include "../include/binary.h"

typedef struct palette_connect_status_t palette_connect_status_t;


// Enum  for palette_connect_status

typedef enum { beater_api_palette_connect_status__NULL = 0, beater_api_palette_connect_status__connected, beater_api_palette_connect_status__waiting_for_trace, beater_api_palette_connect_status__waiting_for_eval, beater_api_palette_connect_status__misconfigured } beater_api_palette_connect_status__e;

char* palette_connect_status_palette_connect_status_ToString(beater_api_palette_connect_status__e palette_connect_status);

beater_api_palette_connect_status__e palette_connect_status_palette_connect_status_FromString(char* palette_connect_status);

cJSON *palette_connect_status_convertToJSON(beater_api_palette_connect_status__e palette_connect_status);

beater_api_palette_connect_status__e palette_connect_status_parseFromJSON(cJSON *palette_connect_statusJSON);

#endif /* _palette_connect_status_H_ */

