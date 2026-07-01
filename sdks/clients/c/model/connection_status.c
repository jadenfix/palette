#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include "connection_status.h"



static connection_status_t *connection_status_create_internal(
    int connected,
    char *connected_account_id,
    char *status,
    char *toolkit
    ) {
    connection_status_t *connection_status_local_var = malloc(sizeof(connection_status_t));
    if (!connection_status_local_var) {
        return NULL;
    }
    connection_status_local_var->connected = connected;
    connection_status_local_var->connected_account_id = connected_account_id;
    connection_status_local_var->status = status;
    connection_status_local_var->toolkit = toolkit;

    connection_status_local_var->_library_owned = 1;
    return connection_status_local_var;
}

__attribute__((deprecated)) connection_status_t *connection_status_create(
    int connected,
    char *connected_account_id,
    char *status,
    char *toolkit
    ) {
    return connection_status_create_internal (
        connected,
        connected_account_id,
        status,
        toolkit
        );
}

void connection_status_free(connection_status_t *connection_status) {
    if(NULL == connection_status){
        return ;
    }
    if(connection_status->_library_owned != 1){
        fprintf(stderr, "WARNING: %s() does NOT free objects allocated by the user\n", "connection_status_free");
        return ;
    }
    listEntry_t *listEntry;
    if (connection_status->connected_account_id) {
        free(connection_status->connected_account_id);
        connection_status->connected_account_id = NULL;
    }
    if (connection_status->status) {
        free(connection_status->status);
        connection_status->status = NULL;
    }
    if (connection_status->toolkit) {
        free(connection_status->toolkit);
        connection_status->toolkit = NULL;
    }
    free(connection_status);
}

cJSON *connection_status_convertToJSON(connection_status_t *connection_status) {
    cJSON *item = cJSON_CreateObject();

    // connection_status->connected
    if (!connection_status->connected) {
        goto fail;
    }
    if(cJSON_AddBoolToObject(item, "connected", connection_status->connected) == NULL) {
    goto fail; //Bool
    }


    // connection_status->connected_account_id
    if(connection_status->connected_account_id) {
    if(cJSON_AddStringToObject(item, "connected_account_id", connection_status->connected_account_id) == NULL) {
    goto fail; //String
    }
    }


    // connection_status->status
    if (!connection_status->status) {
        goto fail;
    }
    if(cJSON_AddStringToObject(item, "status", connection_status->status) == NULL) {
    goto fail; //String
    }


    // connection_status->toolkit
    if (!connection_status->toolkit) {
        goto fail;
    }
    if(cJSON_AddStringToObject(item, "toolkit", connection_status->toolkit) == NULL) {
    goto fail; //String
    }

    return item;
fail:
    if (item) {
        cJSON_Delete(item);
    }
    return NULL;
}

connection_status_t *connection_status_parseFromJSON(cJSON *connection_statusJSON){

    connection_status_t *connection_status_local_var = NULL;

    // connection_status->connected
    cJSON *connected = cJSON_GetObjectItemCaseSensitive(connection_statusJSON, "connected");
    if (cJSON_IsNull(connected)) {
        connected = NULL;
    }
    if (!connected) {
        goto end;
    }

    
    if(!cJSON_IsBool(connected))
    {
    goto end; //Bool
    }

    // connection_status->connected_account_id
    cJSON *connected_account_id = cJSON_GetObjectItemCaseSensitive(connection_statusJSON, "connected_account_id");
    if (cJSON_IsNull(connected_account_id)) {
        connected_account_id = NULL;
    }
    if (connected_account_id) { 
    if(!cJSON_IsString(connected_account_id) && !cJSON_IsNull(connected_account_id))
    {
    goto end; //String
    }
    }

    // connection_status->status
    cJSON *status = cJSON_GetObjectItemCaseSensitive(connection_statusJSON, "status");
    if (cJSON_IsNull(status)) {
        status = NULL;
    }
    if (!status) {
        goto end;
    }

    
    if(!cJSON_IsString(status))
    {
    goto end; //String
    }

    // connection_status->toolkit
    cJSON *toolkit = cJSON_GetObjectItemCaseSensitive(connection_statusJSON, "toolkit");
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


    connection_status_local_var = connection_status_create_internal (
        connected->valueint,
        connected_account_id && !cJSON_IsNull(connected_account_id) ? strdup(connected_account_id->valuestring) : NULL,
        strdup(status->valuestring),
        strdup(toolkit->valuestring)
        );

    return connection_status_local_var;
end:
    return NULL;

}
