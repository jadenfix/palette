#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include "statistical_test.h"


char* statistical_test_statistical_test_ToString(beater_api_statistical_test__e statistical_test) {
    char *statistical_testArray[] =  { "NULL", "paired_t", "mcnemar_exact", "wilcoxon_signed_rank", "paired_bootstrap", "clustered_paired_t", "sequential_e_value" };
    return statistical_testArray[statistical_test];
}

beater_api_statistical_test__e statistical_test_statistical_test_FromString(char* statistical_test) {
    int stringToReturn = 0;
    char *statistical_testArray[] =  { "NULL", "paired_t", "mcnemar_exact", "wilcoxon_signed_rank", "paired_bootstrap", "clustered_paired_t", "sequential_e_value" };
    size_t sizeofArray = sizeof(statistical_testArray) / sizeof(statistical_testArray[0]);
    while(stringToReturn < sizeofArray) {
        if(strcmp(statistical_test, statistical_testArray[stringToReturn]) == 0) {
            return stringToReturn;
        }
        stringToReturn++;
    }
    return 0;
}

cJSON *statistical_test_convertToJSON(beater_api_statistical_test__e statistical_test) {
    cJSON *item = cJSON_CreateObject();
    if(cJSON_AddStringToObject(item, "statistical_test", statistical_test_statistical_test_ToString(statistical_test)) == NULL) {
        goto fail;
    }
    return item;
fail:
    cJSON_Delete(item);
    return NULL;
}

beater_api_statistical_test__e statistical_test_parseFromJSON(cJSON *statistical_testJSON) {
    if(!cJSON_IsString(statistical_testJSON) || (statistical_testJSON->valuestring == NULL)) {
        return 0;
    }
    return statistical_test_statistical_test_FromString(statistical_testJSON->valuestring);
}
