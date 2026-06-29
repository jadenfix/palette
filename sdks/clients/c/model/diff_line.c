#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include "diff_line.h"



static diff_line_t *diff_line_create_internal(
    beater_api_diff_line_kind__e kind,
    int new_line,
    int old_line,
    char *text
    ) {
    diff_line_t *diff_line_local_var = malloc(sizeof(diff_line_t));
    if (!diff_line_local_var) {
        return NULL;
    }
    diff_line_local_var->kind = kind;
    diff_line_local_var->new_line = new_line;
    diff_line_local_var->old_line = old_line;
    diff_line_local_var->text = text;

    diff_line_local_var->_library_owned = 1;
    return diff_line_local_var;
}

__attribute__((deprecated)) diff_line_t *diff_line_create(
    beater_api_diff_line_kind__e kind,
    int new_line,
    int old_line,
    char *text
    ) {
    return diff_line_create_internal (
        kind,
        new_line,
        old_line,
        text
        );
}

void diff_line_free(diff_line_t *diff_line) {
    if(NULL == diff_line){
        return ;
    }
    if(diff_line->_library_owned != 1){
        fprintf(stderr, "WARNING: %s() does NOT free objects allocated by the user\n", "diff_line_free");
        return ;
    }
    listEntry_t *listEntry;
    if (diff_line->text) {
        free(diff_line->text);
        diff_line->text = NULL;
    }
    free(diff_line);
}

cJSON *diff_line_convertToJSON(diff_line_t *diff_line) {
    cJSON *item = cJSON_CreateObject();

    // diff_line->kind
    if (beater_api_diff_line_kind__NULL == diff_line->kind) {
        goto fail;
    }
    cJSON *kind_local_JSON = diff_line_kind_convertToJSON(diff_line->kind);
    if(kind_local_JSON == NULL) {
        goto fail; // custom
    }
    cJSON_AddItemToObject(item, "kind", kind_local_JSON);
    if(item->child == NULL) {
        goto fail;
    }


    // diff_line->new_line
    if(diff_line->new_line) {
    if(cJSON_AddNumberToObject(item, "new_line", diff_line->new_line) == NULL) {
    goto fail; //Numeric
    }
    }


    // diff_line->old_line
    if(diff_line->old_line) {
    if(cJSON_AddNumberToObject(item, "old_line", diff_line->old_line) == NULL) {
    goto fail; //Numeric
    }
    }


    // diff_line->text
    if (!diff_line->text) {
        goto fail;
    }
    if(cJSON_AddStringToObject(item, "text", diff_line->text) == NULL) {
    goto fail; //String
    }

    return item;
fail:
    if (item) {
        cJSON_Delete(item);
    }
    return NULL;
}

diff_line_t *diff_line_parseFromJSON(cJSON *diff_lineJSON){

    diff_line_t *diff_line_local_var = NULL;

    // define the local variable for diff_line->kind
    beater_api_diff_line_kind__e kind_local_nonprim = 0;

    // diff_line->kind
    cJSON *kind = cJSON_GetObjectItemCaseSensitive(diff_lineJSON, "kind");
    if (cJSON_IsNull(kind)) {
        kind = NULL;
    }
    if (!kind) {
        goto end;
    }

    
    kind_local_nonprim = diff_line_kind_parseFromJSON(kind); //custom

    // diff_line->new_line
    cJSON *new_line = cJSON_GetObjectItemCaseSensitive(diff_lineJSON, "new_line");
    if (cJSON_IsNull(new_line)) {
        new_line = NULL;
    }
    if (new_line) { 
    if(!cJSON_IsNumber(new_line))
    {
    goto end; //Numeric
    }
    }

    // diff_line->old_line
    cJSON *old_line = cJSON_GetObjectItemCaseSensitive(diff_lineJSON, "old_line");
    if (cJSON_IsNull(old_line)) {
        old_line = NULL;
    }
    if (old_line) { 
    if(!cJSON_IsNumber(old_line))
    {
    goto end; //Numeric
    }
    }

    // diff_line->text
    cJSON *text = cJSON_GetObjectItemCaseSensitive(diff_lineJSON, "text");
    if (cJSON_IsNull(text)) {
        text = NULL;
    }
    if (!text) {
        goto end;
    }

    
    if(!cJSON_IsString(text))
    {
    goto end; //String
    }


    diff_line_local_var = diff_line_create_internal (
        kind_local_nonprim,
        new_line ? new_line->valuedouble : 0,
        old_line ? old_line->valuedouble : 0,
        strdup(text->valuestring)
        );

    return diff_line_local_var;
end:
    if (kind_local_nonprim) {
        kind_local_nonprim = 0;
    }
    return NULL;

}
