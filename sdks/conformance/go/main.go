// Live conformance: drive the generated Go control-plane client against a
// running beaterd and verify typed request/response shapes. Proves API==SDK for Go.
package main

import (
	"context"
	"fmt"
	"os"

	bc "github.com/GIT_USER_ID/GIT_REPO_ID/beaterclient"
)

func fail(msg string, err error) {
	fmt.Printf("FAIL: %s: %v\n", msg, err)
	os.Exit(1)
}

func main() {
	base := os.Getenv("BEATER_BASE_URL")
	tenant := "demo"
	project := "demo"

	cfg := bc.NewConfiguration()
	cfg.Servers = bc.ServerConfigurations{{URL: base}}
	client := bc.NewAPIClient(cfg)
	ctx := context.Background()

	h, _, err := client.HealthAPI.HealthHealth(ctx).Execute()
	if err != nil {
		fail("health", err)
	}
	fmt.Printf("  health ok=%v\n", h.GetOk())

	_, _, err = client.DatasetsAPI.DatasetsCreateDataset(ctx, tenant, project).
		CreateDatasetRequest(*bc.NewCreateDatasetRequest("conformance-go")).Execute()
	if err != nil {
		fail("createDataset", err)
	}
	fmt.Println("  createDataset -> ok")

	page, _, err := client.TracesAPI.TracesListTraces(ctx, tenant).Execute()
	if err != nil {
		fail("traces.list", err)
	}
	fmt.Printf("  traces.list items=%d\n", len(page.Items))

	fmt.Println("PASS: go generated client round-trips against live API")
}
