import { writable } from "svelte/store";
import { browser } from "$app/environment";

// settings go here
const defaultSettings = {
    testFlag: false,
    coopMode: false,
    selectedPlayer: null,
};

function initClientSettings() {
    let initialSettings = defaultSettings;

    if (browser) {
        const persistedSettings = localStorage.getItem("client-settings");
        if (persistedSettings) {
            try {
                initialSettings = { ...defaultSettings, ...JSON.parse(persistedSettings) };
            } catch (e) {
                console.error("Could not parse client settings from localStorage", e);
                initialSettings = defaultSettings;
            }
        }
    }

    const { subscribe, set, update } = writable(initialSettings);

    return {
        subscribe,
        set,
        update,

        toggleCoopMode: () => {
            update(settings => {
                const updatedSettings = { ...settings, coopMode: !settings.coopMode };
                if (browser) {
                    localStorage.setItem("client-settings", JSON.stringify(updatedSettings));
                }
                return updatedSettings;
            });
        },
        setSelectedPlayer: (uuid) => {
            update(settings => {
                const updatedSettings = { ...settings, selectedPlayerUuid: uuid };
                if (browser) {
                    localStorage.setItem("client-settings", JSON.stringify(updatedSettings));
                }
                return updatedSettings;
            });
        }
    };
}

export const clientSettings = initClientSettings();