package main

import (
	"log"

	"github.com/gofiber/fiber/v2"
	"github.com/gofiber/fiber/v2/middleware/cors"
	"github.com/gofiber/fiber/v2/middleware/logger"
	"github.com/gofiber/fiber/v2/middleware/recover"
	"github.com/rs/zerolog"
	zlog "github.com/rs/zerolog/log"
)

const Version = "0.1.0"

func main() {
	// Configure structured logging
	zerolog.TimeFieldFormat = zerolog.TimeFormatUnix

	app := fiber.New(fiber.Config{
		AppName:      "TGP API Server v" + Version,
		ServerHeader: "TGP",
	})

	// Middleware
	app.Use(recover.New())
	app.Use(logger.New())
	app.Use(cors.New())

	// Routes
	setupRoutes(app)

	// Start server
	zlog.Info().Str("version", Version).Msg("Starting TGP API Server")
	if err := app.Listen(":8080"); err != nil {
		log.Fatalf("Failed to start server: %v", err)
	}
}

func setupRoutes(app *fiber.App) {
	// Health check
	app.Get("/health", func(c *fiber.Ctx) error {
		return c.JSON(fiber.Map{
			"status":  "healthy",
			"version": Version,
		})
	})

	// API v1 routes
	v1 := app.Group("/api/v1")

	// Jobs API
	jobs := v1.Group("/jobs")
	jobs.Post("/submit", submitJob)
	jobs.Get("/:id/status", getJobStatus)
	jobs.Get("/:id/cost", getJobCost)

	// Cluster API
	cluster := v1.Group("/cluster")
	cluster.Get("/status", getClusterStatus)
	cluster.Get("/nodes", getNodes)
}

// Job submission handler
func submitJob(c *fiber.Ctx) error {
	// TODO: Parse job spec and call Rust scheduler via gRPC
	return c.JSON(fiber.Map{
		"job_id": "placeholder",
		"status": "submitted",
	})
}

// Get job status handler
func getJobStatus(c *fiber.Ctx) error {
	jobID := c.Params("id")
	// TODO: Query job status from scheduler
	return c.JSON(fiber.Map{
		"job_id": jobID,
		"status": "running",
	})
}

// Get job cost breakdown handler
func getJobCost(c *fiber.Ctx) error {
	jobID := c.Params("id")
	// TODO: Query cost breakdown from cost calculator
	return c.JSON(fiber.Map{
		"job_id":               jobID,
		"compute_cost_usd":     1.25,
		"data_transfer_usd":    0.15,
		"idle_opportunity_usd": 0.0,
		"total_cost_usd":       1.40,
	})
}

// Get cluster status handler
func getClusterStatus(c *fiber.Ctx) error {
	// TODO: Query cluster status from scheduler
	return c.JSON(fiber.Map{
		"total_nodes":  2,
		"active_nodes": 2,
		"total_jobs":   0,
		"running_jobs": 0,
	})
}

// Get nodes handler
func getNodes(c *fiber.Ctx) error {
	// TODO: Query nodes from scheduler
	return c.JSON([]fiber.Map{
		{
			"id":                "vps-1",
			"location":          "datacenter-1",
			"cpu_cores":         8,
			"memory_gb":         32,
			"gpu_count":         0,
			"cost_per_hour_usd": 0.50,
			"status":            "active",
		},
	})
}
