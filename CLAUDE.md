# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Stablecoin payment platform for crypto-fiat settlement. Focuses on blockchain transaction monitoring, webhook-based payment notifications, and reconciliation between blockchain state and payment databases.

**Current status:** Frontend mobile app with mock data. Backend (Rust) planned.

## Build Commands

### Backend (Rust)
```bash
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

### Backend
Multi-chain stablecoin payment processing system targeting:
- **Solana and Ethereum** support
- **Async processing** using Tokio
- **Idempotent payment handling** for reliability
- **Real-time transaction confirmation** tracking
- **Webhook notifications** for payment events

### Frontend (`frontend/`)
React Native mobile app built with Expo:
- **Expo Router** for file-based navigation
- **NativeWind v4** (Tailwind CSS) for styling
- **TypeScript** throughout
- Screens: Home (balance + recent transactions), Transactions (full history)
- Components in `frontend/components/`, types in `frontend/types/`

Design principles: Abstract blockchain complexity to provide seamless payment UX similar to traditional fintech rails.
