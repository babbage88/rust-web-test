package main

import (
	"fmt"
	"log"
	"math/rand"
	"net/http"
	"sync"
	"time"
)

// randomFloat generates a random float between min and max
func randomFloat(min, max float64) float64 {
	return min + rand.Float64()*(max-min)
}

// randomInt generates a random integer between min and max
func randomInt(min, max int) int {
	return rand.Intn(max-min+1) + min
}

// sendRequest sends a single GET request with random parameters
func sendRequest(client *http.Client, id int, wg *sync.WaitGroup) {
	defer wg.Done()

	// Generate random values for parameters
	initAmount := randomInt(500, 100000)
	monthlyContribution := randomInt(50, 5000)
	interestRate := fmt.Sprintf("%.2f", randomFloat(0.1, 200))
	numberOfYears := randomInt(1, 50)

	// Build the request URL
	url := fmt.Sprintf("https://calc.test.trahan.dev/calculated?initAmount=%d&monthlyContribution=%d&interestRate=%s&numberOfYears=%d",
		initAmount, monthlyContribution, interestRate, numberOfYears)

	// Create the request
	req, err := http.NewRequest("GET", url, nil)
	if err != nil {
		log.Printf("Error creating request %d: %v", id, err)
		return
	}

	// Send the request
	resp, err := client.Do(req)
	if err != nil {
		log.Printf("Error in request %d: %v", id, err)
		return
	}
	defer resp.Body.Close()

	// Log the request details
	log.Printf("Request %d - initAmount: %d, monthlyContribution: %d, interestRate: %s, numberOfYears: %d, Status: %s\n",
		id, initAmount, monthlyContribution, interestRate, numberOfYears, resp.Status)
}

// sendBatch sends a batch of requests with the specified number of concurrent requests
func sendBatch(startID, numRequests int, wg *sync.WaitGroup) {
	client := &http.Client{
		Timeout: 30 * time.Second, // Increased timeout for slower responses
	}

	var batchWG sync.WaitGroup
	for i := 0; i < numRequests; i++ {
		batchWG.Add(1)
		go sendRequest(client, startID+i, &batchWG)
	}
	batchWG.Wait()
	wg.Done()
}

// sendConcurrentRequests breaks requests into batches and sends them 1000 at a time
func sendConcurrentRequests(totalRequests int, batchSize int) {
	var wg sync.WaitGroup
	numBatches := (totalRequests + batchSize - 1) / batchSize // Calculate number of batches

	for batch := 0; batch < numBatches; batch++ {
		wg.Add(1)
		startID := batch * batchSize
		requestsInBatch := min(batchSize, totalRequests-startID)
		go sendBatch(startID, requestsInBatch, &wg)
		wg.Wait() // Wait for the current batch to finish before starting the next
	}
	wg.Wait() // Ensure all batches are completed
}

// min function to get the minimum value
func min(a, b int) int {
	if a < b {
		return a
	}
	return b
}
