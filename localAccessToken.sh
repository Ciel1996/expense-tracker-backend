#!/bin/bash

# Configuration variables
URL="http://localhost:8080/realms/local/protocol/openid-connect/token"
CLIENT_ID="expense-tracker"
CLIENT_SECRET="kPEFrXZAe8QiFKG8bRvgWHcPisaXGdyH"
USERNAME="test"
PASSWORD="test"

# Requesting the token
RESPONSE=$(curl -s -X POST "$URL" \
    -H 'Content-Type: application/x-form-urlencoded' \
    -d "client_id=$CLIENT_ID" \
    -d "client_secret=$CLIENT_SECRET" \
    -d "username=$USERNAME" \
    -d "password=$PASSWORD" \
    -d "grant_type=password" \
    -d "scope=email profile")

# Extracting the access token using jq
TOKEN=$(echo "$RESPONSE" | jq -r '.access_token')

if [ "$TOKEN" != "null" ] && [ -n "$TOKEN" ]; then
    # Copying to clipboard
    if [[ "$OSTYPE" == "darwin"* ]]; then
        # macOS
        echo -n "$TOKEN" | pbcopy
        echo "Access token copied to macOS clipboard."
    elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
        # Linux
        if command -v xclip > /dev/null; then
            echo -n "$TOKEN" | xclip -selection clipboard
            echo "Access token copied to Linux clipboard (xclip)."
        elif command -v wl-copy > /dev/null; then
            echo -n "$TOKEN" | wl-copy
            echo "Access token copied to Linux clipboard (wl-copy)."
        else
            echo "Token found but no clipboard tool (pbcopy, xclip, wl-copy) found."
            echo "Token: $TOKEN"
        fi
    fi
else
    echo "Failed to retrieve access token. Response:"
    echo "$RESPONSE" | jq .
fi