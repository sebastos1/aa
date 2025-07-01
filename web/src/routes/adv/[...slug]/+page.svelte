<script>
	import { page } from "$app/stores";
	import { advancements, players, categories } from "$lib/stores.js";

	const advancementId = $derived($page.params.slug);
	
	const advancement = $derived($advancements[advancementId]);
	const category = $derived(advancement?.category ? $categories[advancement.category] : null);
	
	const firstPlayerUuid = $derived(Object.keys($players)[0]);
	const firstPlayer = $derived(firstPlayerUuid ? $players[firstPlayerUuid] : null);
	const progress = $derived(
		advancement && firstPlayer ? firstPlayer.advancement_progress?.[advancementId] : null
	);
</script>

<svelte:head>
	<title>Debug: {advancement?.displayName || advancementId}</title>
</svelte:head>

<div class="debug-container">
	<h1>Advancement Debug</h1>
	
    <div class="debug-section">
        <h2>Basic Info</h2>
        <div class="field">
            <strong>Key:</strong> <code>{advancement.key}</code>
        </div>
        <div class="field">
            <strong>Display Name:</strong> {advancement.displayName}
        </div>
        <div class="field">
            <strong>Description:</strong> {advancement.description}
        </div>
        <div class="field">
            <strong>Source:</strong> <code>{advancement.source}</code>
        </div>
        <div class="field">
            <strong>Type:</strong> <code>{advancement.advancementType}</code>
        </div>
    </div>

    <div class="debug-section">
        <h2>Hierarchy</h2>
        <div class="field">
            <strong>Parent:</strong> 
            {#if advancement.parent}
                <a href="/adv/{advancement.parent}"><code>{advancement.parent}</code></a>
            {:else}
                <em>None (Root)</em>
            {/if}
        </div>
        <div class="field">
            <strong>Category:</strong> <code>{advancement.category}</code>
            {#if category}
                <span>({category.displayName})</span>
            {/if}
        </div>
    </div>

    <div class="debug-section">
        <h2>Icon</h2>
        <div class="field">
            <strong>Type:</strong> <code>{Object.keys(advancement.icon)[0]}</code>
        </div>
        <div class="field">
            <strong>Data:</strong> <code>{JSON.stringify(advancement.icon, null, 2)}</code>
        </div>
    </div>

    <div class="debug-section">
        <h2>Requirements</h2>
        {#if advancement.requirements && Object.keys(advancement.requirements).length > 0}
            {#each Object.entries(advancement.requirements) as [reqKey, requirements]}
                <div class="requirement">
                    <strong>{reqKey}:</strong>
                    <ul>
                        {#each requirements as requirement}
                            <li><code>{JSON.stringify(requirement, null, 2)}</code></li>
                        {/each}
                    </ul>
                </div>
            {/each}
        {:else}
            <em>No requirements</em>
        {/if}
    </div>

    <div class="debug-section">
        <h2>Spreadsheet Info</h2>
        <div class="field">
            <strong>Class:</strong> <code>{advancement.spreadsheetInfo?.class || "None"}</code>
        </div>
        <div class="field">
            <strong>Requirement Details:</strong> 
            {#if advancement.spreadsheetInfo?.requirementDetails}
                <code>{advancement.spreadsheetInfo.requirementDetails}</code>
            {:else}
                <em>None</em>
            {/if}
        </div>
    </div>

    {#if progress}
        <div class="debug-section">
            <h2>Progress ({firstPlayer?.name || firstPlayerUuid})</h2>
            <div class="field">
                <strong>Done:</strong> <code>{progress.done}</code>
            </div>
            <div class="field">
                <strong>Requirements:</strong>
                {#if progress.requirementProgress && Object.keys(progress.requirementProgress).length > 0}
                    <ul>
                        {#each Object.entries(progress.requirementProgress) as [requirementKey, date]}
                            <li><code>{requirementKey}</code>: {date}</li>
                        {/each}
                    </ul>
                {:else}
                    <em>No requirements completed</em>
                {/if}
            </div>
        </div>
    {:else}
        <div class="debug-section">
            <h2>Progress</h2>
            <em>No progress data for {firstPlayer?.name || firstPlayerUuid || "unknown player"}</em>
        </div>
    {/if}

    <div class="debug-section">
        <h2>Raw JSON</h2>
        <pre><code>{JSON.stringify(advancement, null, 2)}</code></pre>
    </div>
</div>

<style>
	.debug-container {
		max-width: 800px;
		margin: 2rem auto;
		padding: 2rem;
		font-family: "minecraft", monospace;
		background: #1a1a1a;
		color: #eee;
	}

	.debug-section {
		margin-bottom: 2rem;
		padding: 1rem;
		background: #2a2a2a;
		border-radius: 4px;
	}

	.debug-section h2 {
		margin: 0 0 1rem 0;
		color: #4a9eff;
		font-size: 1.2rem;
	}

	.field {
		margin-bottom: 0.5rem;
		line-height: 1.4;
	}

	.requirement {
		margin-bottom: 1rem;
	}

	.requirement ul {
		margin: 0.5rem 0 0 1rem;
		padding: 0;
	}

	.requirement li {
		margin-bottom: 0.25rem;
	}

	code {
		background: #333;
		padding: 0.2rem 0.4rem;
		border-radius: 2px;
		font-family: "Courier New", monospace;
		color: #ff6b6b;
	}

	pre {
		background: #333;
		padding: 1rem;
		border-radius: 4px;
		overflow-x: auto;
		white-space: pre-wrap;
	}

	pre code {
		background: none;
		padding: 0;
		color: #eee;
	}

	a {
		color: #4a9eff;
		text-decoration: none;
	}

	a:hover {
		text-decoration: underline;
	}

	h1 {
		text-align: center;
		color: #4a9eff;
		margin-bottom: 2rem;
	}
</style>