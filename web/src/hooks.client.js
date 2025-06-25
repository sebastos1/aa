import { players } from '$lib/stores';

console.log('Client hook running: setting up EventSource...');

const eventSource = new EventSource('/api/events');

eventSource.onmessage = (event) => {
    const update = JSON.parse(event.data);
    $players[update.uuid] = update.player;
};

eventSource.onerror = (err) => {
    console.error('EventSource connection failed:', err);
};