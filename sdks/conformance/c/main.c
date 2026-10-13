/*
 * Live conformance for the GENERATED C control-plane client.
 *
 * This links the generated client sources directly (HealthAPI + apiClient +
 * cJSON + health_response model) and drives them against a live beaterd:
 *   - GET /health via the generated HealthAPI_healthHealth() -> typed health_response_t
 *
 * createDataset/listTraces are exercised over raw libcurl below (clearly
 * labeled): the generated DatasetsAPI/TracesAPI sources transitively include
 * model files that hit pre-existing openapi-generator C codegen bugs
 * (array-of-enum models such as api_key_created_response.c reference
 * non-existent helpers). Those models are unrelated to this round-trip, so we
 * keep the generated-client check honest (real SDK code for health) and prove
 * the rest of the API end-to-end over raw HTTP from the same C program.
 */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <curl/curl.h>

#include "apiClient.h"
#include "HealthAPI.h"
#include "health_response.h"

struct buf { char *data; size_t len; };

static size_t sink(void *ptr, size_t sz, size_t nm, void *ud) {
    struct buf *b = (struct buf *)ud;
    size_t n = sz * nm;
    b->data = realloc(b->data, b->len + n + 1);
    memcpy(b->data + b->len, ptr, n);
    b->len += n;
    b->data[b->len] = '\0';
    return n;
}

/* Raw libcurl POST helper; returns HTTP status (0 on transport error). */
static long http_post_json(const char *url, const char *body, struct buf *out) {
    CURL *c = curl_easy_init();
    if (!c) return 0;
    struct curl_slist *hdr = curl_slist_append(NULL, "Content-Type: application/json");
    curl_easy_setopt(c, CURLOPT_URL, url);
    curl_easy_setopt(c, CURLOPT_POSTFIELDS, body);
    curl_easy_setopt(c, CURLOPT_HTTPHEADER, hdr);
    curl_easy_setopt(c, CURLOPT_WRITEFUNCTION, sink);
    curl_easy_setopt(c, CURLOPT_WRITEDATA, out);
    CURLcode rc = curl_easy_perform(c);
    long code = 0;
    if (rc == CURLE_OK) curl_easy_getinfo(c, CURLINFO_RESPONSE_CODE, &code);
    curl_slist_free_all(hdr);
    curl_easy_cleanup(c);
    return code;
}

static long http_get(const char *url, struct buf *out) {
    CURL *c = curl_easy_init();
    if (!c) return 0;
    curl_easy_setopt(c, CURLOPT_URL, url);
    curl_easy_setopt(c, CURLOPT_WRITEFUNCTION, sink);
    curl_easy_setopt(c, CURLOPT_WRITEDATA, out);
    CURLcode rc = curl_easy_perform(c);
    long code = 0;
    if (rc == CURLE_OK) curl_easy_getinfo(c, CURLINFO_RESPONSE_CODE, &code);
    curl_easy_cleanup(c);
    return code;
}

int main(void) {
    const char *base = getenv("BEATER_BASE_URL");
    if (!base) { fprintf(stderr, "FAIL: BEATER_BASE_URL unset\n"); return 1; }
    const char *tenant = getenv("BEATER_TENANT");  if (!tenant)  tenant = "demo";
    const char *project = getenv("BEATER_PROJECT"); if (!project) project = "demo";

    curl_global_init(CURL_GLOBAL_DEFAULT);

    /* ---- GENERATED CLIENT: typed GET /health ---- */
    apiClient_t *api = apiClient_create_with_base_path(base, NULL);
    if (!api) { fprintf(stderr, "FAIL: apiClient_create\n"); return 1; }
    health_response_t *h = HealthAPI_healthHealth(api);
    if (!h) {
        fprintf(stderr, "FAIL: HealthAPI_healthHealth returned NULL (status=%ld)\n",
                api->response_code);
        return 1;
    }
    if (h->ok != 1) { fprintf(stderr, "FAIL: health ok != true (%d)\n", h->ok); return 1; }
    printf("  [generated client] health ok=%d\n", h->ok);
    health_response_free(h);
    apiClient_free(api);

    /* ---- RAW HTTP (build-verify fallback for codegen-broken model graph) ---- */
    char url[512];
    struct buf rb = {0};

    snprintf(url, sizeof url, "%s/v1/datasets/%s/%s", base, tenant, project);
    long code = http_post_json(url, "{\"name\":\"conformance-c\"}", &rb);
    if (code < 200 || code >= 300) {
        fprintf(stderr, "FAIL: createDataset status=%ld body=%s\n", code, rb.data ? rb.data : "");
        return 1;
    }
    if (!rb.data || !strstr(rb.data, "conformance-c")) {
        fprintf(stderr, "FAIL: createDataset response missing name: %s\n", rb.data ? rb.data : "(empty)");
        return 1;
    }
    printf("  [raw http]         createDataset -> %ld ok\n", code);
    free(rb.data);

    struct buf tb = {0};
    snprintf(url, sizeof url, "%s/v1/traces/%s", base, tenant);
    code = http_get(url, &tb);
    if (code < 200 || code >= 300) {
        fprintf(stderr, "FAIL: listTraces status=%ld body=%s\n", code, tb.data ? tb.data : "");
        return 1;
    }
    if (!tb.data || !strstr(tb.data, "items")) {
        fprintf(stderr, "FAIL: listTraces response missing items: %s\n", tb.data ? tb.data : "(empty)");
        return 1;
    }
    printf("  [raw http]         listTraces -> %ld ok\n", code);
    free(tb.data);

    curl_global_cleanup();
    printf("PASS: c generated HealthAPI round-trips live; createDataset+listTraces verified over raw HTTP\n");
    return 0;
}
