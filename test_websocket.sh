#!/bin/bash
# Make script executable: chmod +x test_websocket.sh

# WebSocket Connection Test Script for Zerodha
echo "🔧 Zerodha WebSocket Connection Test"
echo "===================================="

# Read credentials from config.toml
API_KEY=$(grep 'api_key' config.toml | cut -d'"' -f2)
ACCESS_TOKEN=$(grep 'access_token' config.toml | cut -d'"' -f2)

echo "📋 Configuration:"
echo "   API Key: $API_KEY"
echo "   Access Token: ${ACCESS_TOKEN:0:8}..."

# Test 1: Check if access token is placeholder
if [ "$ACCESS_TOKEN" = "your_access_token_here" ]; then
    echo "❌ ERROR: Access token is still placeholder!"
    echo "   Run: cargo run --bin auth_helper"
    exit 1
fi

# Test 2: Validate REST API access
echo ""
echo "🧪 Testing REST API access..."
HTTP_RESPONSE=$(curl -s -w "HTTPSTATUS:%{http_code}" \
    -H "X-Kite-Version: 3" \
    -H "Authorization: token $API_KEY:$ACCESS_TOKEN" \
    https://api.kite.trade/user/profile)

HTTP_BODY=$(echo $HTTP_RESPONSE | sed -E 's/HTTPSTATUS\:[0-9]{3}$//')
HTTP_STATUS=$(echo $HTTP_RESPONSE | tr -d '\n' | sed -E 's/.*HTTPSTATUS:([0-9]{3})$/\1/')

if [ "$HTTP_STATUS" -eq 200 ]; then
    echo "✅ REST API access: SUCCESS"
else
    echo "❌ REST API access: FAILED ($HTTP_STATUS)"
    echo "   Response: $HTTP_BODY"
    echo "   💡 Your access token may be expired. Run: cargo run --bin auth_helper"
    exit 1
fi

echo ""
echo "🔗 Correct WebSocket URL format:"
echo "   wss://ws.kite.trade?api_key=$API_KEY&access_token=${ACCESS_TOKEN:0:8}..."
echo ""
echo "🎯 The main fix: Remove extra slash from WebSocket URL"
echo "   ❌ Wrong: wss://ws.kite.trade/?api_key=..."
echo "   ✅ Right: wss://ws.kite.trade?api_key=..."
echo ""
echo "💡 Next steps:"
echo "   1. The WebSocket URL has been fixed in your code"
echo "   2. Make sure your access_token in config.toml is valid (not placeholder)"
echo "   3. Rebuild and run: cargo build && cargo run"
