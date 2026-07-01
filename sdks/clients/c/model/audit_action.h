/*
 * audit_action.h
 *
 * 
 */

#ifndef _audit_action_H_
#define _audit_action_H_

#include <string.h>
#include "../external/cJSON.h"
#include "../include/list.h"
#include "../include/keyValuePair.h"
#include "../include/binary.h"

typedef struct audit_action_t audit_action_t;


// Enum  for audit_action

typedef enum { beater_api_audit_action__NULL = 0, beater_api_audit_action__pii_unmask, beater_api_audit_action__api_key_create, beater_api_audit_action__api_key_revoke, beater_api_audit_action__provider_secret_create, beater_api_audit_action__provider_secret_revoke } beater_api_audit_action__e;

char* audit_action_audit_action_ToString(beater_api_audit_action__e audit_action);

beater_api_audit_action__e audit_action_audit_action_FromString(char* audit_action);

cJSON *audit_action_convertToJSON(beater_api_audit_action__e audit_action);

beater_api_audit_action__e audit_action_parseFromJSON(cJSON *audit_actionJSON);

#endif /* _audit_action_H_ */

