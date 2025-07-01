import { players, progress } from '$lib/stores';

console.log("Client hook running: setting up EventSource...");

const eventSource = new EventSource("/api/events");

eventSource.onmessage = (event) => {
    const update = JSON.parse(event.data);

    players.update(currentPlayers => {
        currentPlayers[update.uuid] = update.player;
        return currentPlayers;
    });

    progress.update(currentProgress => {

        for (const advancementKey in currentProgress) {
            if (currentProgress[advancementKey][update.uuid]) {
                delete currentProgress[advancementKey][update.uuid];
            }
        }

        // update each new advancement
        console.log(currentProgress);

        for (const [advancementKey, progressDetails] of Object.entries(update.updatedProgress)) {
            if (!currentProgress[advancementKey]) currentProgress[advancementKey] = {};
            currentProgress[advancementKey][update.uuid] = progressDetails;
        }
        return currentProgress;
    });
};

eventSource.onerror = (err) => {
    console.error("EventSource connection failed:", err);
};