#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include "connector_skills_response.h"



static connector_skills_response_t *connector_skills_response_create_internal(
    char *skills,
    char *toolkit
    ) {
    connector_skills_response_t *connector_skills_response_local_var = malloc(sizeof(connector_skills_response_t));
    if (!connector_skills_response_local_var) {
        return NULL;
    }
    connector_skills_response_local_var->skills = skills;
    connector_skills_response_local_var->toolkit = toolkit;

    connector_skills_response_local_var->_library_owned = 1;
    return connector_skills_response_local_var;
}

__attribute__((deprecated)) connector_skills_response_t *connector_skills_response_create(
    char *skills,
    char *toolkit
    ) {
    return connector_skills_response_create_internal (
        skills,
        toolkit
        );
}

void connector_skills_response_free(connector_skills_response_t *connector_skills_response) {
    if(NULL == connector_skills_response){
        return ;
    }
    if(connector_skills_response->_library_owned != 1){
        fprintf(stderr, "WARNING: %s() does NOT free objects allocated by the user\n", "connector_skills_response_free");
        return ;
    }
    listEntry_t *listEntry;
    if (connector_skills_response->skills) {
        free(connector_skills_response->skills);
        connector_skills_response->skills = NULL;
    }
    if (connector_skills_response->toolkit) {
        free(connector_skills_response->toolkit);
        connector_skills_response->toolkit = NULL;
    }
    free(connector_skills_response);
}

cJSON *connector_skills_response_convertToJSON(connector_skills_response_t *connector_skills_response) {
    cJSON *item = cJSON_CreateObject();

    // connector_skills_response->skills
    if (!connector_skills_response->skills) {
        goto fail;
    }
    if(cJSON_AddStringToObject(item, "skills", connector_skills_response->skills) == NULL) {
    goto fail; //String
    }


    // connector_skills_response->toolkit
    if (!connector_skills_response->toolkit) {
        goto fail;
    }
    if(cJSON_AddStringToObject(item, "toolkit", connector_skills_response->toolkit) == NULL) {
    goto fail; //String
    }

    return item;
fail:
    if (item) {
        cJSON_Delete(item);
    }
    return NULL;
}

connector_skills_response_t *connector_skills_response_parseFromJSON(cJSON *connector_skills_responseJSON){

    connector_skills_response_t *connector_skills_response_local_var = NULL;

    // connector_skills_response->skills
    cJSON *skills = cJSON_GetObjectItemCaseSensitive(connector_skills_responseJSON, "skills");
    if (cJSON_IsNull(skills)) {
        skills = NULL;
    }
    if (!skills) {
        goto end;
    }

    
    if(!cJSON_IsString(skills))
    {
    goto end; //String
    }

    // connector_skills_response->toolkit
    cJSON *toolkit = cJSON_GetObjectItemCaseSensitive(connector_skills_responseJSON, "toolkit");
    if (cJSON_IsNull(toolkit)) {
        toolkit = NULL;
    }
    if (!toolkit) {
        goto end;
    }

    
    if(!cJSON_IsString(toolkit))
    {
    goto end; //String
    }


    connector_skills_response_local_var = connector_skills_response_create_internal (
        strdup(skills->valuestring),
        strdup(toolkit->valuestring)
        );

    return connector_skills_response_local_var;
end:
    return NULL;

}
