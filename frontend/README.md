# Last.fm Dating App - Frontend

React + TypeScript + Vite frontend for the Last.fm dating application.

## Setup

1. Install dependencies:
```bash
npm install
```

2. Create a `.env` file (optional):
```bash
VITE_API_BASE_URL=http://localhost:8000
```

3. Run the development server:
```bash
npm run dev
```

The app will be available at http://localhost:3000

## Build

```bash
npm run build
```

## Features

- **Authentication**: Register and login with email/password
- **Last.fm Integration**: Connect your Last.fm account
- **Discover**: Swipe through profiles with music compatibility scores
- **Matches**: View your matches and compatibility scores
- **Profile**: Manage your profile and Last.fm connection

## Tech Stack

- React 18
- TypeScript
- Vite
- TailwindCSS
- Zustand (state management)
- React Query (data fetching)
- React Router (routing)
- Axios (HTTP client)

## Project Structure

```
src/
├── components/     # React components
│   ├── ui/        # Base UI components
│   ├── discover/  # Discover page components
│   ├── layout/    # Layout components
│   └── ...
├── pages/         # Page components
├── api/           # API client
├── hooks/         # Custom hooks
├── stores/        # Zustand stores
├── types/         # TypeScript types
└── styles/        # Global styles
```
