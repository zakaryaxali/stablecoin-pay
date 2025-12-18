# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Stablecoin payment platform for crypto-fiat settlement. Focuses on blockchain transaction monitoring, webhook-based payment notifications, and reconciliation between blockchain state and payment databases.

**Current status:** Frontend mobile app with mock data. Backend (Rust) API ready.

## Build Commands

### Backend (Rust)
```bash
cd backend
cargo build              # Build
cargo test               # Run tests
cargo test <test_name>   # Run single test
cargo build --release    # Release build
cargo check              # Check without building
cargo fmt                # Format code
cargo clippy             # Lint code
```

### Frontend (React Native / Expo)
```bash
cd frontend
npm start                # Start Expo dev server
npm run ios              # Run on iOS simulator
npm run android          # Run on Android emulator
npm run web              # Run in browser
```

## Architecture

### Backend (`backend/`)
Rust REST API for Solana wallet tracking:
- **Axum** web framework with **Tokio** async runtime
- **PostgreSQL** with SQLx (compile-time checked queries)
- **Solana RPC** integration for balance and transaction fetching
- Modules: `api/` (handlers), `domain/` (models), `repository/` (DB), `services/` (Solana client)

**API Endpoints:**
- `POST /wallets` - Register wallet to track
- `GET /wallets/:address/balance` - Get USDC balance
- `GET /wallets/:address/transactions` - Get transaction history
- `GET /health` - Health check

**Environment:** Copy `backend/.env.example` to `backend/.env` and configure DATABASE_URL.

### Frontend (`frontend/`)
React Native mobile app built with Expo:
- **Expo Router** for file-based navigation
- **NativeWind v4** (Tailwind CSS) for styling
- **TypeScript** throughout
- Screens: Home (balance + recent transactions), Transactions (full history)
- Components in `frontend/components/`, types in `frontend/types/`

Design principles: Abstract blockchain complexity to provide seamless payment UX similar to traditional fintech rails.
