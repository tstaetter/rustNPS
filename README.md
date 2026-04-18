# rustNPS

API for Net Promoter Score (NPS) management, built with Rust and Axum.

## Overview

rustNPS is a microservice designed to handle NPS entries and dismissals. It provides a RESTful API to record user
feedback and manage notification states.

## Tech Stack

- **Language:** [Rust](https://www.rust-lang.org/) (Edition 2024)
- **Web Framework:** [Axum](https://github.com/tokio-rs/axum)
- **Runtime:** [Tokio](https://tokio.rs/)
- **Serialization:** [Serde](https://serde.rs/), [BSON](https://github.com/mongodb/bson-rust)
- **Database:** [MongoDB](https://github.com/mongodb/mongodb-rust-driver) (Driver included)
- **API Specification:** OpenAPI 3.1.1 (Redocly for linting)

## Requirements

- **Rust:** `1.85.0` or later (recommended)
- **Node.js & npm:** For OpenAPI linting and documentation tools

## Setup & Run

### Environment Variables

The application uses `dotenvy` to load configuration. Create a `.env` file in the root directory:

```env
RUST_LOG=rust_nps=debug,tower_http=debug
# TODO: Add MongoDB connection string once implemented
# MONGODB_URI=mongodb://localhost:27017
```

### Running the Application

To start the server on `http://0.0.0.0:3000`:

```bash
cargo run
```

### OpenAPI Linting

To lint the OpenAPI specification:

```bash
npm install
npm run lint
```

## Scripts

- `cargo run`: Starts the API server.
- `cargo test`: Runs Rust unit and integration tests.
- `npm run lint`: Lints the `openapi_v1.yaml` file using Redocly.
- `npm test`: Alias for `npm run lint`.

## Project Structure

```text
.
├── Cargo.toml          # Rust dependencies and metadata
├── package.json        # Node.js scripts and devDependencies
├── openapi_v1.yaml     # API specification
├── src/
│   ├── main.rs         # Application entry point and server setup
│   ├── lib.rs          # Library root, app factory, and shared types
│   ├── routes.rs       # Axum router definitions
│   ├── handlers.rs     # Request handlers (business logic)
│   ├── payloads.rs     # Data transfer objects (DTOs) and BSON models
│   ├── error.rs        # Custom error types and handling
│   └── segment.rs      # Segmentation logic
└── target/             # Compiled artifacts
```

## API Endpoints

The API is versioned under `/v1`.

- `POST /v1/nps`: Create a new NPS entry.
- `DELETE /v1/nps/dismiss`: Dismiss an NPS entry.

See `openapi_v1.yaml` for full details.

## Tests

### Rust Tests

Run the test suite using:

```bash
cargo test
```

The project uses `axum-test` for integration testing.

### OpenAPI Linting

```bash
npm run lint
```

## License

This project is licensed under the [MIT License](LICENSE).
