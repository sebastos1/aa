export async function load({ fetch }) {
    console.log('Fetching bootstrap data in layout load function...');
    try {
        const response = await fetch('/api/init');
        const data = await response.json();

        return {
            advancements: data.advancements,
            players: data.players,
            categories: data.categories,
            world: data.world
        };
    } catch (error) {
        console.error('Failed to load bootstrap data:', error);
        return { advancements: {}, players: {}, categories: {}, world: {} };
    }
}