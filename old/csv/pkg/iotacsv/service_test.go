package iotacsv_test

import (
	"context"
	"io"
	"os"
	"testing"

	"github.com/alecthomas/assert"
	"github.com/candlecorp/iotacsv/pkg/iotacsv"
	"github.com/nanobus/iota/go/rx/flux"
	"github.com/stretchr/testify/require"
)

func TestParse(t *testing.T) {
	ctx := context.Background()
	reader, err := os.Open("testdata/data.csv")
	require.NoError(t, err)
	defer reader.Close()

	csvParser := iotacsv.NewCSV(iotacsv.Dependencies{})

	fBytes := flux.NewProcessor[[]byte]()
	fBytes.OnSubscribe(flux.OnSubscribe{
		Request: func(n int) {
			for {
				var buf [10]byte
				n, err := reader.Read(buf[:])
				if err == io.EOF {
					fBytes.Complete()
					break
				}
				require.NoError(t, err)
				if n > 0 {
					fBytes.Next(buf[0:n])
				} else if n == 0 {
					fBytes.Complete()
				}
			}
		},
	})

	done := make(chan struct{})
	records := []map[string]string{}
	parse := csvParser.Parse(ctx, &iotacsv.Options{
		Delimiter:     ",",
		Quote_char:    "\"",
		Row_separator: "\n",
		Escape_char:   "\\",
		Header_row:    false,
	}, fBytes)
	go parse.Subscribe(flux.Subscribe[map[string]string]{
		OnNext: func(v map[string]string) {
			records = append(records, v)
		},
		OnComplete: func() {
			close(done)
		},
		OnError: func(err error) {
			t.Fail()
		},
	})

	<-done

	require.Equal(t, 3, len(records))
	assert.Equal(t, "Visa", records[2]["4"])
	assert.Equal(t, "12/31/1970  5:00:00", records[2]["13"])
}
