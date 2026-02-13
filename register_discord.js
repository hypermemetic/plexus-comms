const WebSocket = require('ws');

const ws = new WebSocket('ws://127.0.0.1:4445');

ws.on('open', () => {
    console.log('Connected to plexus-comms server');
    
    const request = {
        activation: 'discord',
        method: 'register_account',
        params: {
            name: 'my-bot',
            bot_token: 'REDACTED_DISCORD_BOT_TOKEN'
        }
    };
    
    console.log('Sending request:', JSON.stringify(request, null, 2));
    ws.send(JSON.stringify(request));
});

ws.on('message', (data) => {
    console.log('Received:', data.toString());
});

ws.on('error', (error) => {
    console.error('WebSocket error:', error);
});

ws.on('close', () => {
    console.log('Connection closed');
    process.exit(0);
});

setTimeout(() => {
    ws.close();
}, 5000);
