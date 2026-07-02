#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include "signature.h"



static signature_t *signature_create_internal(
    char *hash,
    list_t *shingles
    ) {
    signature_t *signature_local_var = malloc(sizeof(signature_t));
    if (!signature_local_var) {
        return NULL;
    }
    signature_local_var->hash = hash;
    signature_local_var->shingles = shingles;

    signature_local_var->_library_owned = 1;
    return signature_local_var;
}

__attribute__((deprecated)) signature_t *signature_create(
    char *hash,
    list_t *shingles
    ) {
    return signature_create_internal (
        hash,
        shingles
        );
}

void signature_free(signature_t *signature) {
    if(NULL == signature){
        return ;
    }
    if(signature->_library_owned != 1){
        fprintf(stderr, "WARNING: %s() does NOT free objects allocated by the user\n", "signature_free");
        return ;
    }
    listEntry_t *listEntry;
    if (signature->hash) {
        free(signature->hash);
        signature->hash = NULL;
    }
    if (signature->shingles) {
        list_ForEach(listEntry, signature->shingles) {
            free(listEntry->data);
        }
        list_freeList(signature->shingles);
        signature->shingles = NULL;
    }
    free(signature);
}

cJSON *signature_convertToJSON(signature_t *signature) {
    cJSON *item = cJSON_CreateObject();

    // signature->hash
    if (!signature->hash) {
        goto fail;
    }
    if(cJSON_AddStringToObject(item, "hash", signature->hash) == NULL) {
    goto fail; //String
    }


    // signature->shingles
    if (!signature->shingles) {
        goto fail;
    }
    cJSON *shingles = cJSON_AddArrayToObject(item, "shingles");
    if(shingles == NULL) {
        goto fail; //primitive container
    }

    listEntry_t *shinglesListEntry;
    list_ForEach(shinglesListEntry, signature->shingles) {
    if(cJSON_AddStringToObject(shingles, "", shinglesListEntry->data) == NULL)
    {
        goto fail;
    }
    }

    return item;
fail:
    if (item) {
        cJSON_Delete(item);
    }
    return NULL;
}

signature_t *signature_parseFromJSON(cJSON *signatureJSON){

    signature_t *signature_local_var = NULL;

    // define the local list for signature->shingles
    list_t *shinglesList = NULL;

    // signature->hash
    cJSON *hash = cJSON_GetObjectItemCaseSensitive(signatureJSON, "hash");
    if (cJSON_IsNull(hash)) {
        hash = NULL;
    }
    if (!hash) {
        goto end;
    }

    
    if(!cJSON_IsString(hash))
    {
    goto end; //String
    }

    // signature->shingles
    cJSON *shingles = cJSON_GetObjectItemCaseSensitive(signatureJSON, "shingles");
    if (cJSON_IsNull(shingles)) {
        shingles = NULL;
    }
    if (!shingles) {
        goto end;
    }

    
    cJSON *shingles_local = NULL;
    if(!cJSON_IsArray(shingles)) {
        goto end;//primitive container
    }
    shinglesList = list_createList();

    cJSON_ArrayForEach(shingles_local, shingles)
    {
        if(!cJSON_IsString(shingles_local))
        {
            goto end;
        }
        list_addElement(shinglesList , strdup(shingles_local->valuestring));
    }


    signature_local_var = signature_create_internal (
        strdup(hash->valuestring),
        shinglesList
        );

    return signature_local_var;
end:
    if (shinglesList) {
        listEntry_t *listEntry = NULL;
        list_ForEach(listEntry, shinglesList) {
            free(listEntry->data);
            listEntry->data = NULL;
        }
        list_freeList(shinglesList);
        shinglesList = NULL;
    }
    return NULL;

}
