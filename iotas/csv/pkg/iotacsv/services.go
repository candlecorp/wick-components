package iotacsv

import (
	"bytes"
	"context"
	"fmt"
	"strconv"

	"github.com/nanobus/iota/go/rx/flux"
)

type CSVImpl struct {
}

func NewCSV(deps Dependencies) *CSVImpl {
	return &CSVImpl{}
}

func (c *CSVImpl) Parse(ctx context.Context, options *Options, input flux.Flux[[]byte]) flux.Flux[map[string]string] {
	return flux.Create(func(sink flux.Sink[map[string]string]) {
		var buffer bytes.Buffer
		var inQuote bool
		var inEscape bool
		var row []string

		input.Subscribe(flux.Subscribe[[]byte]{
			OnNext: func(v []byte) {
				for _, b := range v {
					if b == options.Escape_char[0] {
						inEscape = true
					} else if inEscape {
						buffer.WriteByte(b)
						inEscape = false
						continue
					}
					if b == options.Quote_char[0] {
						if inQuote {
							// end quote
							inQuote = false
						} else {
							// start quote
							inQuote = true
						}
					}

					if !inQuote && b == options.Delimiter[0] {
						element := buffer.String()
						//remove leading and trailing quotes
						if len(element) > 0 && element[0] == options.Quote_char[0] {
							element = element[1:]
						}
						if len(element) > 0 && element[len(element)-1] == options.Quote_char[0] {
							element = element[:len(element)-1]
						}

						row = append(row, element)
						buffer.Reset()
					} else if !inQuote && b != options.Row_separator[0] {
						buffer.WriteByte(b)
					} else if !inQuote && b == options.Row_separator[0] {
						element := buffer.String()
						//remove leading and trailing quotes
						if len(element) > 0 && element[0] == options.Quote_char[0] {
							element = element[1:]
						}
						if len(element) > 0 && element[len(element)-1] == options.Quote_char[0] {
							element = element[:len(element)-1]
						}
						//remove tailing \r
						if len(element) > 0 && element[len(element)-1] == '\r' {
							element = element[:len(element)-1]
						}

						row = append(row, element)

						// loop through row and add to map
						parsedLine := make(map[string]string)
						for i, element := range row {
							parsedLine[strconv.Itoa(i)] = element
						}
						fmt.Println(parsedLine)
						sink.Next(parsedLine)
						row = []string{}
						buffer.Reset()
					} else {
						buffer.WriteByte(b)
					}
				}
			},
			OnError: func(err error) {
				sink.Error(err)
			},
			OnComplete: func() {
				sink.Complete()
			},
		})
	})
}
