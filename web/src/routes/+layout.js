export const ssr = false

export async function load({ fetch }) {
    console.log('Fetching bootstrap data in load function...');
    
    try {
        const response = await fetch(`/api/init`);

        console.log("ASDFADSF");
        console.log(response);

        if (!response.ok) {
            console.error('API response not ok:', response.status, response.statusText);
            return { advancements: {}, players: {}, categories: {}, world: {} };
        }

        return await response.json();
    } catch (error) {
        console.error('Failed to load bootstrap data:', error);
        return { advancements: {}, players: {}, categories: {}, world: {} };
    }
}