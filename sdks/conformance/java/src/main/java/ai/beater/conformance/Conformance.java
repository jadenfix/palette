package ai.beater.conformance;

import ai.beater.client.ApiClient;
import ai.beater.client.api.DatasetsApi;
import ai.beater.client.api.HealthApi;
import ai.beater.client.api.TracesApi;
import ai.beater.client.model.CreateDatasetRequest;
import ai.beater.client.model.Dataset;
import ai.beater.client.model.HealthResponse;
import ai.beater.client.model.PageRunSummary;

/**
 * Live conformance: drive the GENERATED Java control-plane client against a
 * running beaterd and verify typed request/response shapes. Proves API==SDK
 * for Java.
 */
public final class Conformance {
    public static void main(String[] args) {
        String base = System.getenv("BEATER_BASE_URL");
        if (base == null || base.isEmpty()) {
            System.err.println("FAIL: BEATER_BASE_URL unset");
            System.exit(1);
        }
        String tenant = envOr("BEATER_TENANT", "demo");
        String project = envOr("BEATER_PROJECT", "demo");

        ApiClient client = new ApiClient();
        // This generated client uses java.net.http and splits scheme/host/port;
        // updateBaseUri parses all of them (setBasePath only sets the path).
        client.updateBaseUri(base);

        try {
            HealthResponse health = new HealthApi(client).healthHealth();
            if (health.getOk() == null || !health.getOk()) {
                fail("health ok != true: " + health);
            }
            System.out.println("  health ok=" + health.getOk());

            CreateDatasetRequest req = new CreateDatasetRequest().name("conformance-java");
            // Optional auth/context headers are null in local auth mode.
            Dataset ds = new DatasetsApi(client)
                    .datasetsCreateDataset(tenant, project, req, null, null, null, null);
            if (ds == null) {
                fail("createDataset returned null");
            }
            System.out.println("  createDataset -> ok (" + ds.getName() + ")");

            PageRunSummary page = new TracesApi(client).tracesListTraces(
                    tenant, null, null, null, null, null, null, null, null, null,
                    null, null, null, null, null, null, null, null, null, null);
            int items = page.getItems() == null ? 0 : page.getItems().size();
            System.out.println("  traces.list items=" + items);

            System.out.println("PASS: java generated client round-trips against live API");
        } catch (Exception e) {
            fail(e.toString());
        }
    }

    private static String envOr(String k, String d) {
        String v = System.getenv(k);
        return (v == null || v.isEmpty()) ? d : v;
    }

    private static void fail(String msg) {
        System.err.println("FAIL: " + msg);
        System.exit(1);
    }
}
