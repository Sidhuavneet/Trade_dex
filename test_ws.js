const WebSocket = require('ws');

const ws = new WebSocket('ws://localhost:3000/ws/trades');

ws.on('open', () => {
    console.log('✅ WebSocket connected!');
});

ws.on('message', (data) => {
    const trade = JSON.parse(data.toString());
    console.log('📊 Trade received:', trade);
});

ws.on('error', (error) => {
    console.error('❌ WebSocket error:', error);
});

ws.on('close', () => {
    console.log('🔌 WebSocket closed');
});