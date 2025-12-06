# Last.fm Dating App - Backend

Rust backend using Axum framework for the Last.fm dating application.

## Setup

1. Install Rust (https://rustup.rs/)

2. Copy `.env.example` to `.env` and fill in the required values:
```bash
cp .env.example .env
```

3. Set up MySQL database and run migrations:
```bash
# Create database
mysql -u root -p -e "CREATE DATABASE lastfm_dating;"

# Run migrations
mysql -u root -p lastfm_dating < migrations/001_initial_schema.sql
```

4. Get Last.fm API credentials:
   - Go to https://www.last.fm/api/account/create
   - Create an API application
   - Copy the API Key and Shared Secret to your `.env` file

5. Build and run:
```bash
cargo build
cargo run
```

The server will start on http://localhost:8000

## API Endpoints

### Authentication
- `POST /auth/register` - Register new user
- `POST /auth/login` - Login
- `POST /auth/logout` - Logout

### Users
- `GET /users/me` - Get current user (auth required)
- `PUT /users/me` - Update current user (auth required)
- `GET /users/:id` - Get user by ID

### Last.fm
- `POST /lastfm/connect` - Connect Last.fm account (auth required)
- `POST /lastfm/sync` - Sync scrobbles from Last.fm (auth required)

### Discover
- `GET /discover` - Get potential matches (auth required)

### Matches
- `POST /likes` - Like a user (auth required)
- `GET /matches` - Get all matches (auth required)
- `DELETE /matches/:id` - Delete a match (auth required)

### Photos
- `POST /photos` - Add a photo (auth required)
- `GET /photos/:user_id` - Get user's photos
- `DELETE /photos/:id` - Delete a photo (auth required)

## Development

```bash
# Run in watch mode
cargo watch -x run

# Run tests
cargo test

# Format code
cargo fmt

# Lint code
cargo clippy
```
