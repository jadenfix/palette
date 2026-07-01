#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include "connection_link.h"



static connection_link_t *connection_link_create_internal(
    char *connected_account_id,
    char *expires_at,
    char *redirect_url
    ) {
    connection_link_t *connection_link_local_var = malloc(sizeof(connection_link_t));
    if (!connection_link_local_var) {
        return NULL;
    }
    connection_link_local_var->connected_account_id = connected_account_id;
    connection_link_local_var->expires_at = expires_at;
    connection_link_local_var->redirect_url = redirect_url;

    connection_link_local_var->_library_owned = 1;
    return connection_link_local_var;
}

__attribute__((deprecated)) connection_link_t *connection_link_create(
    char *connected_account_id,
    char *expires_at,
    char *redirect_url
    ) {
    return connection_link_create_internal (
        connected_account_id,
        expires_at,
        redirect_url
        );
}

void connection_link_free(connection_link_t *connection_link) {
    if(NULL == connection_link){
        return ;
    }
    if(connection_link->_library_owned != 1){
        fprintf(stderr, "WARNING: %s() does NOT free objects allocated by the user\n", "connection_link_free");
        return ;
    }
    listEntry_t *listEntry;
    if (connection_link->connected_account_id) {
        free(connection_link->connected_account_id);
        connection_link->connected_account_id = NULL;
    }
    if (connection_link->expires_at) {
        free(connection_link->expires_at);
        connection_link->expires_at = NULL;
    }
    if (connection_link->redirect_url) {
        free(connection_link->redirect_url);
        connection_link->redirect_url = NULL;
    }
    free(connection_link);
}

cJSON *connection_link_convertToJSON(connection_link_t *connection_link) {
    cJSON *item = cJSON_CreateObject();

    // connection_link->connected_account_id
    if (!connection_link->connected_account_id) {
        goto fail;
    }
    if(cJSON_AddStringToObject(item, "connected_account_id", connection_link->connected_account_id) == NULL) {
    goto fail; //String
    }


    // connection_link->expires_at
    if(connection_link->expires_at) {
    if(cJSON_AddStringToObject(item, "expires_at", connection_link->expires_at) == NULL) {
    goto fail; //String
    }
    }


    // connection_link->redirect_url
    if (!connection_link->redirect_url) {
        goto fail;
    }
    if(cJSON_AddStringToObject(item, "redirect_url", connection_link->redirect_url) == NULL) {
    goto fail; //String
    }

    return item;
fail:
    if (item) {
        cJSON_Delete(item);
    }
    return NULL;
}

connection_link_t *connection_link_parseFromJSON(cJSON *connection_linkJSON){

    connection_link_t *connection_link_local_var = NULL;

    // connection_link->connected_account_id
    cJSON *connected_account_id = cJSON_GetObjectItemCaseSensitive(connection_linkJSON, "connected_account_id");
    if (cJSON_IsNull(connected_account_id)) {
        connected_account_id = NULL;
    }
    if (!connected_account_id) {
        goto end;
    }

    
    if(!cJSON_IsString(connected_account_id))
    {
    goto end; //String
    }

    // connection_link->expires_at
    cJSON *expires_at = cJSON_GetObjectItemCaseSensitive(connection_linkJSON, "expires_at");
    if (cJSON_IsNull(expires_at)) {
        expires_at = NULL;
    }
    if (expires_at) { 
    if(!cJSON_IsString(expires_at) && !cJSON_IsNull(expires_at))
    {
    goto end; //String
    }
    }

    // connection_link->redirect_url
    cJSON *redirect_url = cJSON_GetObjectItemCaseSensitive(connection_linkJSON, "redirect_url");
    if (cJSON_IsNull(redirect_url)) {
        redirect_url = NULL;
    }
    if (!redirect_url) {
        goto end;
    }

    
    if(!cJSON_IsString(redirect_url))
    {
    goto end; //String
    }


    connection_link_local_var = connection_link_create_internal (
        strdup(connected_account_id->valuestring),
        expires_at && !cJSON_IsNull(expires_at) ? strdup(expires_at->valuestring) : NULL,
        strdup(redirect_url->valuestring)
        );

    return connection_link_local_var;
end:
    return NULL;

}
