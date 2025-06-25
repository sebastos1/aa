<script>
	import { DEFAULTS } from '$lib/utils.js';
	import PlayerIcon from './PlayerIcon.svelte';

	let { players, selectedPlayerUuid, onSelect } = $props();

	let isOpen = $state(false);

	const selectedPlayer = $derived(players.find(p => p.uuid === selectedPlayerUuid));
	const otherPlayers = $derived(players.filter((p) => p.uuid !== selectedPlayerUuid));

	function handleSelect(uuid) {
		onSelect(uuid);
		isOpen = false;
	}
</script>

<div class="custom-dropdown" onmouseleave={() => (isOpen = false)}>
	<button
		class="selected-item-button"
		class:open={isOpen}
		onmouseenter={() => (isOpen = true)}
	>
		<PlayerIcon player={selectedPlayer} />
	</button>

	{#if isOpen && otherPlayers.length > 0}
		<ul class="options-list">
			{#each otherPlayers as player (player.uuid)}
				<li>
					<button class="option-item" onclick={() => handleSelect(player.uuid)}>
						<PlayerIcon {player} />
					</button>
				</li>
			{/each}
		</ul>
	{/if}
</div>

<style>
	.custom-dropdown {
		position: relative;
		display: inline-block;
		font-size: inherit;
		font-family: 'minecraft', monospace;
		vertical-align: middle;
	}

	.selected-item-button {
		display: inline-flex;
		align-items: center;
		background-color: #3a3a3a;
		border-radius: 6px;
		cursor: pointer;
		color: #eee;
		width: max-content;
	}

	.options-list {
		position: absolute;
		top: 100%;
		list-style: none;
		z-index: 10;
		min-width: 100%;
	}

	.option-item {
		display: flex;
		width: max-content;
        margin-bottom: 4px;
		background: #3a3a3a;
		border-radius: 6px;
		text-align: left;
		cursor: pointer;
	}

	.option-item:hover {
		background-color: #4f4f4f;
	}

	.selected-item-button > :global(.icon-text-container),
	.option-item > :global(.icon-text-container) {
		padding: 0.25rem 0.5rem;
		width: 100%;
	}
</style>