# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Architecture Overview

This is a Rust microservice built with Axum that handles Net Promoter Score (NPS) feedback. The application:

- **Entry Point**: `src/main.rs` initializes MongoDB connection and starts the Axum server on port 8000
- **Application Factory**: `src/lib.rs` creates the Axum router with shared `AppState` (MongoDB database reference)
- **Module Structure**:
  - `src/routes.rs`: Defines Axum routes (GET/POST /v1/nps, DELETE /v1/nps/dismiss)
  - `src/handlers/`: Business logic organized in modules (`create`, `dismiss`, `index`, `stats`)
  - `src/payloads.rs`: DTOs with validation (validator crate)
  - `src/db/`: MongoDB model (`NpsEntry`)
  - `src/segment.rs`: Segment enum (User, Studio, Professional)
  - `src/error.rs`: Custom error types wrapping MongoDB and I/O errors

## Common Commands

```bash
# Run the application
cargo run

# Run tests
cargo test

# Run a single test
cargo test -- --test-threads=1 --exact <test_name>

# Lint OpenAPI spec
npm run lint

# Install dependencies
npm install
```

## Environment Setup

Create `.env` file in project root:
```env
RUST_LOG=rust_nps=debug,tower_http=debug
MONGODB_URI=mongodb://localhost:27017
MONGODB_DB=rust_nps
```

## API Endpoints

All endpoints are under `/v1` prefix and require JWT bearer token authentication.

- `POST /v1/nps` - Create NPS entry (payload: user ObjectId, segment, score, optional comment)
- `DELETE /v1/nps/dismiss` - Dismiss an entry (payload: user ObjectId, segment, dismissed)
- `GET /v1/nps` - Get dashboard with stats (query param: `period` in days, default 90)

## Key Implementation Details

### NPS Calculation
- Promoters: score >= 9
- Passives: score 7-8
- Detractors: score <= 6
- NPS = (% promoters - % detractors), rounded to integer

### Segment Types
Enum `Segment` supports: User, Studio, Professional (defaults to User)

### Data Flow
1. Request arrives at router (`src/routes.rs`)
2. Handler extracts state and payload (`src/handlers/*.rs`)
3. Validation via `validator` crate derives
4. MongoDB operations via `mongodb` crate collections
5. Responses use `axum::Json` with status codes

### Testing
- Integration tests use `axum-test` crate for mocking requests
- Tests run against in-memory or actual MongoDB instance

## Validation Rules

- ObjectId fields use custom validation function
- All payloads derive `Validate` from validator crate
- Validation errors return 422 UNPROCESSABLE_ENTITY
