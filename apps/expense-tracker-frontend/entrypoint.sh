#!/bin/sh
set -e

# Replace placeholder with actual runtime value
# Find all JS files and replace the placeholder
find /app -type f -name "*.js" -exec sed -i "s|__NEXT_PUBLIC_API_URL__|${NEXT_PUBLIC_API_URL:-http://localhost:3001}|g" {} \;

# Execute the main command
exec "$@"