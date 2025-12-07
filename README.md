# Last.fm Dating App 🎵💕

A modern dating application that connects people based on their music taste using Last.fm data. Find your perfect match through shared musical interests!

## Overview

This application uses Last.fm scrobbles to calculate musical compatibility between users. The matching algorithm considers:
- Common artists between users
- Popularity of common artists (niche artists = better match)
- Similarity in listening preferences

## Stack

### Backend (Rust)
- **Framework**: Axum (web framework)
- **Database**: MySQL with SQLx
- **Authentication**: JWT with Argon2 password hashing
- **APIs**: Last.fm API integration
- **Storage**: Prepared for S3/MinIO (photo uploads)

### Frontend (TypeScript)
- **Framework**: React 18 + Vite
- **Styling**: TailwindCSS
- **State Management**: Zustand
- **Data Fetching**: TanStack Query (React Query)
- **Routing**: React Router v6

## Features

### ✅ Implemented
- User registration and authentication
- Last.fm account connection
- Profile management
- Music compatibility algorithm
- Discover profiles with swipe interface
- Match system (mutual likes)
- Photo management (up to 6 photos)
- Responsive design (mobile-first)

### 🔒 Security Features
- JWT Bearer token authentication
- Argon2 password hashing
- Protected API routes
- SQL injection prevention
- CORS restrictions
- Secure password handling (never exposed)
- Environment-based configuration

See [SECURITY.md](SECURITY.md) for complete security documentation.

### 🚧 Coming Soon
- Real-time chat (WebSocket)
- Advanced filters (distance, age, gender)
- Push notifications
- Photo upload to S3/MinIO

## Getting Started

### Prerequisites
- Rust (https://rustup.rs/)
- Node.js 18+ and npm
- MySQL 8+
- Last.fm API credentials (https://www.last.fm/api/account/create)

### Backend Setup

1. Navigate to the backend directory:
```bash
cd backend
```

2. Copy the example environment file:
```bash
cp .env.example .env
```

3. Edit `.env` and fill in your configuration:
```env
DATABASE_URL=mysql://user:password@localhost:3306/lastfm_dating
JWT_SECRET=your-super-secret-key-change-this-in-production
LASTFM_API_KEY=your-lastfm-api-key
LASTFM_API_SECRET=your-lastfm-api-secret
ALLOWED_ORIGINS=http://localhost:3000
```

⚠️ **Security Note**: Use a strong JWT_SECRET in production (minimum 32 random characters)

4. Create the database and run migrations:
```bash
mysql -u root -p -e "CREATE DATABASE lastfm_dating;"
mysql -u root -p lastfm_dating < migrations/001_initial_schema.sql
```

5. Build and run the backend:
```bash
cargo run
```

The API will be available at http://localhost:8000

### Frontend Setup

1. Navigate to the frontend directory:
```bash
cd frontend
```

2. Install dependencies:
```bash
npm install
```

3. Run the development server:
```bash
npm run dev
```

The app will be available at http://localhost:3000

## API Documentation

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

### Discover & Matches
- `GET /discover` - Get potential matches (auth required)
- `POST /likes` - Like a user (auth required)
- `GET /matches` - Get all matches (auth required)
- `DELETE /matches/:id` - Delete a match (auth required)

### Photos
- `POST /photos` - Add a photo (auth required)
- `GET /photos/:user_id` - Get user's photos
- `DELETE /photos/:id` - Delete a photo (auth required)

## Compatibility Algorithm

The matching algorithm calculates a score from 0-100% based on:

1. **Common Artists** (30 points max): Number of shared artists
2. **Niche Factor** (70 points max): Weight inversely proportional to artist popularity
3. **Rank Similarity**: How similar the artist rankings are

Formula:
```rust
weighted_score = Σ (popularity_weight × position_weight)
popularity_weight = 1 / log10(avg_listeners)
position_weight = 1 / (1 + position_diff / 10)
total_score = (common_count_score × 0.3) + (weighted_score × 0.7)
```

## Database Schema

See `backend/migrations/001_initial_schema.sql` for the complete schema.

Main tables:
- `users` - User accounts and profiles
- `photos` - User photos (up to 6 per user)
- `scrobbles_cache` - Cached Last.fm data
- `likes` - User likes (swipe right)
- `matches` - Mutual likes
- `messages` - Chat messages
- `blocks` - Blocked users

## Development

### Backend
```bash
cd backend
cargo watch -x run    # Watch mode
cargo test           # Run tests
cargo fmt            # Format code
cargo clippy         # Lint code
```

### Frontend
```bash
cd frontend
npm run dev          # Development server
npm run build        # Production build
npm run lint         # Lint code
```

## UI Design

Inspired by Duolicious, the UI features:
- Dark mode first (#0F0F0F background)
- Purple (#8B5CF6) and pink (#EC4899) accent colors
- Mobile-first responsive design
- Card-based swipe interface
- Bottom navigation for mobile
- Top navbar for desktop

## License

MIT

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Roadmap

- [ ] WebSocket integration for real-time chat
- [ ] Push notifications
- [ ] S3/MinIO photo storage
- [ ] Advanced filtering (location, age, preferences)
- [ ] Social features (profile sharing, etc.)
- [ ] Mobile apps (React Native)
- [ ] AI-powered conversation starters based on music taste