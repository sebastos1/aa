<script>
	import { PLACEHOLDERS } from "$lib/utils.js";
	import PlayerIcon from "./PlayerIcon.svelte";
	import { clientSettings } from "$lib/clientSettings.js";
	import { players, selectedPlayer } from "$lib/stores.js";

	let isOpen = $state(false);

	const otherPlayers = $derived(
        Object.values($players).filter((p) => p.uuid !== $clientSettings.selectedPlayerUuid)
    );

	function handleSelect(uuid) {
		clientSettings.setSelectedPlayer(uuid);
		isOpen = false;
	}
</script>

<div class="dropdown" onmouseleave={() => (isOpen = false)}>
	<button class="selected-item" class:open={isOpen} onmouseenter={() => (isOpen = true)} >
		<PlayerIcon player={$selectedPlayer} />
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
	* {
		box-sizing: border-box;
	}

	.dropdown {
		position: relative;
		display: inline-block;
		font-size: inherit;
		font-family: "minecraft", monospace;
	}

	.selected-item {
		display: inline-flex;
		align-items: center;
		vertical-align: middle;
		background-color: #3a3a3a;
		border-radius: 6px;
		border: 2px solid #6d6d6d;
		cursor: pointer;
		color: #eee;
		width: max-content;
		padding: 0.25rem 0.5rem;
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
		background: #3a3a3a;
		border-radius: 6px;
		text-align: left;
		cursor: pointer;
		padding: 0.25rem 0.5rem;
		border: 2px solid #6d6d6d;
	}

	.option-item:hover {
		background-color: #4f4f4f;
	}
</style>