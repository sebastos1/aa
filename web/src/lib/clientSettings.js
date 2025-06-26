import { writable } from 'svelte/store';
import { browser } from '$app/environment';

// settings go here
const defaultSettings = {
    testFlag: false
};

function setCookie(name, value) {
    if (!browser) return;
    const d = new Date();
    d.setTime(d.getTime() + (365 * 24 * 60 * 60 * 1000)); // Expires in 1 year
    const expires = "expires=" + d.toUTCString();
    document.cookie = `${name}=${JSON.stringify(value)};${expires};path=/;SameSite=Lax`;
}

function saveToLocalStorage(settings) {
    if (!browser) return;
    localStorage.setItem('client-settings', JSON.stringify(settings));
}

function createClientSettingsStore() {
    const { subscribe, set, update } = writable(defaultSettings);

    return {
        set,
        subscribe,
        init: (initialSettings) => {
            set(initialSettings);
        },

        toggleTestFlag: () => {
            update(settings => {
                const updatedSettings = { ...settings, testFlag: !settings.testFlag };

                // save to cookie and localstorage!
                saveToLocalStorage(updatedSettings);
                setCookie('client-settings', updatedSettings);

                return updatedSettings;
            });
        }
    };
}

export const clientSettings = createClientSettingsStore();

// On initial client-side load, check localStorage to hydrate the store.
// This is important for client-side navigations where load doesn't re-run.
if (browser) {
    const persistedSettings = localStorage.getItem('client-settings');
    if (persistedSettings) {
        try {
            clientSettings.init(JSON.parse(persistedSettings));
        } catch (e) {
            console.error("Could not parse client settings from localStorage", e);
        }
    }
}