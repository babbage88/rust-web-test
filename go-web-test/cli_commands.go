package main

import (
	"strconv"
	"time"

	"github.com/urfave/cli/v2"
)

func SendPostRequests() (appInst *cli.App) {
	appInst = &cli.App{
		Name:                 "gowebtest",
		Version:              "0.0.1",
		Compiled:             time.Now(),
		Args:                 true,
		EnableBashCompletion: true,
		Authors: []*cli.Author{
			{
				Name:  "Justin Trahan",
				Email: "justin@trahan.dev",
			},
		},
		Flags: []cli.Flag{
			&cli.IntFlag{
				Name:    "requests",
				Aliases: []string{"n"},
				Usage:   "The Total Number of requests to send.",
			},
			&cli.IntFlag{
				Name:    "batch-size",
				Aliases: []string{"b"},
				Value:   75,
				Usage:   "batch size, number of request to send at a time.",
			},
		},
		Action: func(cCtx *cli.Context) error {
			var totalRequests, batchSize int

			// Check if the flags are set, otherwise use positional arguments
			if cCtx.IsSet("requests") {
				totalRequests = cCtx.Int("requests")
			} else if cCtx.Args().Len() > 0 {
				arg0, err := strconv.Atoi(cCtx.Args().Get(0))
				if err != nil {
					return cli.Exit("Invalid value for total requests. Expected an integer.", 1)
				}
				totalRequests = arg0
			}

			if cCtx.IsSet("batch-size") {
				batchSize = cCtx.Int("batch-size")
			} else if cCtx.Args().Len() > 1 {
				arg1, err := strconv.Atoi(cCtx.Args().Get(1))
				if err != nil {
					return cli.Exit("Invalid value for batch size. Expected an integer.", 1)
				}
				batchSize = arg1
			}

			if totalRequests == 0 || batchSize == 0 {
				return cli.Exit("Please provide valid values for total requests and batch size.", 1)
			}

			// Call the function to send the requests
			sendConcurrentRequests(totalRequests, batchSize)
			return nil
		},
	}
	return appInst
}
