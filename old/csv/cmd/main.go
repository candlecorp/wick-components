package main

import (
	"github.com/nanobus/iota/go/transport/wasmrs/guest"

	"github.com/candlecorp/iotacsv/pkg/iotacsv"
)

func main() {
	// Create providers
	deps := iotacsv.GetDependencies(guest.HostInvoker)

	// Create services
	csvService := iotacsv.NewCSV(deps)

	// Register services
	iotacsv.RegisterCSV(csvService)
}
