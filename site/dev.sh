#!/bin/bash
# Simplified development script

# Clean up previous installations
rm -rf node_modules package-lock.json

# Install only essential dependencies
npm install astro@latest @astrojs/svelte@latest svelte@latest

# Run dev server
npm run dev
