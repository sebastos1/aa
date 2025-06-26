<script>
	import Advancement from '$lib/components/Advancement.svelte';
	import Header from '$lib/components/Header.svelte';
	import { clientSettings } from '$lib/clientSettings.js';

	let { data } = $props();
	const { advancements, players, categories, world } = data;

	let selectedPlayerUuid = $state(Object.keys(players)[0] ?? '');
	const selectedPlayer = $derived(players[selectedPlayerUuid]);
	const advancementList = $derived(Object.values(advancements));

	function handlePlayerChange(newUuid) {
		selectedPlayerUuid = newUuid;
	}
</script>

<Header {world} players={Object.values(players)} {selectedPlayerUuid} onPlayerChange={handlePlayerChange} />

<main>
	<div class="test-flag-container">
		<div 
			class="toggle-button" 
			onclick={() => clientSettings.toggleCoopMode()}
			role="button"
			tabindex="0"
		>
			CLICK HERE TO TOGGLE
		</div>
		<p>
			Current state: <strong>{$clientSettings.toggleCoopMode ? "ON" : "OFF"}</strong>
		</p>
		{#if $clientSettings.toggleCoopMode}
			<p>TTHE FLAG IS ON, wave it</p>
		{/if}
	</div>

	<div class="advancement-grid">
		{#each advancementList as advancement (advancement.key)}
			{@const progress = selectedPlayer?.advancement_progress[advancement.key]}
			{@const category = (data.categories || {})[advancement.category]}
			<Advancement {advancement} {progress} {category} showTitle={false} />
		{/each}
	</div>

</main>

<style>
	main {
		width: 100%;
		max-width: 75rem;
		margin: 0 auto;
	}
	
	.advancement-grid {
		display: flex;
		flex-wrap: wrap;
		gap: 0.5rem;
		justify-content: center;
	}
</style>