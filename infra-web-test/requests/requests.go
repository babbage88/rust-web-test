package requests

import (
	"log"
	"log/slog"
	"math/rand"
	"net/http"
	"strings"
	"sync"
	"time"
)

type AuthenticatedClient struct {
	Client  *http.Client  `json:"httpClient"`
	Request *http.Request `json:"httpRequest"`
	Error   error         `json:"error"`
}

func NewAuthenticatedClientRequest(authKey string, uri string, method string, timeoutSec int) AuthenticatedClient {
	var authenticatedClientReq AuthenticatedClient
	var authHdrBearer strings.Builder
	authHdrBearer.WriteString("Bearer ")
	authHdrBearer.WriteString(authKey)
	c := http.Client{Timeout: time.Duration(timeoutSec) * time.Second}
	req, err := http.NewRequest(method, uri, nil)
	if err != nil {
		slog.Error("error initializing request", slog.String("error", err.Error()))
		authenticatedClientReq.Error = err
	}
	req.Header.Add("Accept", "application/json")
	req.Header.Add("Authorization", authHdrBearer.String())
	authenticatedClientReq.Client = &c
	authenticatedClientReq.Request = req
	return authenticatedClientReq
}

// randomFloat generates a random float between min and max
func randomFloat(min, max float64) float64 {
	return min + rand.Float64()*(max-min)
}

// randomInt generates a random integer between min and max
func randomInt(min, max int) int {
	return rand.Intn(max-min+1) + min
}

// sendRequest sends a single GET request with random parameters
func sendRequest(cr AuthenticatedClient, id int, wg *sync.WaitGroup) {
	defer wg.Done()

	// Send the request
	resp, err := cr.Client.Do(cr.Request)
	if err != nil {
		log.Printf("Error in request %d: %v", id, err)
		return
	}
	defer resp.Body.Close()

	// Log the request details
	log.Printf("RequestID %d - Status: %s\n", id, resp.Status)
}

// sendBatch sends a batch of requests with the specified number of concurrent requests
func sendBatch(startID, numRequests, timeoutSec int, authKey, uri, method string, wg *sync.WaitGroup) {
	clientReq := NewAuthenticatedClientRequest(authKey, uri, method, timeoutSec)
	var batchWG sync.WaitGroup
	for i := 0; i < numRequests; i++ {
		batchWG.Add(1)
		go sendRequest(clientReq, startID+i, &batchWG)
	}
	batchWG.Wait()
	wg.Done()
}

// sendConcurrentRequests breaks requests into batches and sends them 1000 at a time
func sendConcurrentRequests(totalRequests, timeoutSec, batchSize int, uri, authKey, method string) {
	var wg sync.WaitGroup
	numBatches := (totalRequests + batchSize - 1) / batchSize // Calculate number of batches

	for batch := 0; batch < numBatches; batch++ {
		wg.Add(1)
		startID := batch * batchSize
		requestsInBatch := min(batchSize, totalRequests-startID)
		go sendBatch(startID, requestsInBatch, timeoutSec, authKey, uri, method, &wg)
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
