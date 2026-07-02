/*
 * failure_mode.h
 *
 * A deterministically-inferred class of failure.
 */

#ifndef _failure_mode_H_
#define _failure_mode_H_

#include <string.h>
#include "../external/cJSON.h"
#include "../include/list.h"
#include "../include/keyValuePair.h"
#include "../include/binary.h"

typedef struct failure_mode_t failure_mode_t;


// Enum  for failure_mode

typedef enum { beater_api_failure_mode__NULL = 0, beater_api_failure_mode__tool_error, beater_api_failure_mode__timeout, beater_api_failure_mode__guardrail_block, beater_api_failure_mode__wrong_output, beater_api_failure_mode__retrieval_miss, beater_api_failure_mode__other } beater_api_failure_mode__e;

char* failure_mode_failure_mode_ToString(beater_api_failure_mode__e failure_mode);

beater_api_failure_mode__e failure_mode_failure_mode_FromString(char* failure_mode);

cJSON *failure_mode_convertToJSON(beater_api_failure_mode__e failure_mode);

beater_api_failure_mode__e failure_mode_parseFromJSON(cJSON *failure_modeJSON);

#endif /* _failure_mode_H_ */

