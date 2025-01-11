package main

import (
	"log"
	"os"
)

func main() {
	app := SendPostRequests()
	if err := app.Run(os.Args); err != nil {
		log.Fatal(err)
	}
}
