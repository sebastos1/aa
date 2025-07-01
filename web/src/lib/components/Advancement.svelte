<script>
	import { getIconUrl, getFrameUrl, formatDate, getAdvancementTextColor } from "$lib/utils.js";
	import { players, progress } from "$lib/stores.js";

    // only specific advancement data, player specific progress
	let { advancement, selectedPlayerUuid, category, showTitle = true } = $props();



	const playerProgress = $derived($progress[advancement.key]?.[selectedPlayerUuid]);
	const isCompleted = $derived(playerProgress?.done ?? false);
	const completionTime = $derived(isCompleted ? Object.values(playerProgress.requirementProgress)[0] : undefined);
	const totalRequirements = $derived(Object.keys(advancement.requirements ?? {}).length);
    const progressCount = $derived(Object.keys(playerProgress?.requirementProgress ?? {}).length);
	const isInProgress = $derived(!isCompleted && progressCount > 0 && totalRequirements > 1);

    const iconUrl = $derived(getIconUrl(advancement.icon));
    const categoryIconUrl = $derived(getIconUrl(category?.icon));
    const frameUrl = $derived(getFrameUrl(advancement.advancementType, isCompleted));
    const titleColor = $derived(getAdvancementTextColor(advancement.advancementType));
	const descriptionText = $derived(advancement.spreadsheetInfo.requirementDetails
		? `${advancement.description} (${advancement.spreadsheetInfo.requirementDetails})` 
		: advancement.description
	);
</script>

<!-- {#if isInProgress || isCompleted } -->
<a href={`/adv/${advancement.key}`} class="advancement" style:--title-color={titleColor}>
	<div class="icon" style:background-image="url('{frameUrl}')">
		<img src={iconUrl} alt={advancement.displayName} />
	</div>

	<div class="title">{progressCount}</div>
	<div class="title">{totalRequirements}</div>

	{#if showTitle}
		<div class="title">{advancement.displayName}</div>
	{/if}

	<div class="hover-box">
		<div class="hover-header">
			<div class="hover-icon-space"></div>
			<div class="hover-title">{advancement.displayName}</div>
		</div>
		<div class="hover-description">{descriptionText}</div>

        {#if isCompleted && completionTime}
            <div class="hover-status" style:--status-color="limegreen">
                Achieved: {formatDate(completionTime)}
            </div>
        {:else if isInProgress}
            <div class="hover-status" style:--status-color="yellow">
                Achieved ({progressCount}/{totalRequirements})
            </div>
        {/if}

		<div class="hover-meta">
			{#if category}
				<div class="clickable">
					<img src={categoryIconUrl} alt="category" />
					{category.displayName}
				</div>
			{/if}
			<span>{advancement.source}</span>
		</div>
	</div>
</a>
<!-- {/if} -->

<style>
	a.advancement {
		text-decoration: none;
		color: inherit;
		text-align: center;
		cursor: pointer;
		position: relative;
		z-index: 1;
		display: inline-block;
	}
	.advancement:hover {
		z-index: 1000;
	}
	.advancement:hover .hover-box {
		display: block;
	}
	.advancement:hover .icon {
		z-index: 1002;
	}
	.icon {
		width: 65px;
		height: 65px;
		background-size: contain;
		display: flex;
		align-items: center;
		justify-content: center;
		margin: 0 auto;
		position: relative;
	}
	img {
		width: 32px;
		height: 32px;
		image-rendering: pixelated;
	}
	.title {
		font-family: "minecraft", monospace;
		line-height: 1.2;
		display: flex;
		justify-content: center;
		color: var(--title-color);
	}
	.hover-box {
		position: absolute;
		bottom: 110%;
		left: 50%;
		transform: translateX(-50%);
		width: 16rem;
		background: rgb(20, 20, 20);
		padding: 0.5rem;
		display: none;
		z-index: 1001;
		border: 1px solid;
		border-radius: 10px;
		border-color: var(--title-color);
	}
	.hover-header {
		display: flex;
		align-items: center;
	}
	.hover-title {
		font-weight: bold;
		font-size: 1.2rem;
		color: var(--title-color);
		flex: 1;
		text-align: left;
	}
	.hover-description {
		line-height: 1.3;
		margin-bottom: 0.5rem;
		text-align: left;
		display: flex;
		color: #fff;
	}
	.hover-icon-space {
		display: none;
	}
	.hover-status {
		color: var(--status-color);
		font-style: italic;
		margin-bottom: 0.5rem;
		text-align: left;
	}
	.hover-meta {
		display: flex;
		justify-content: space-between;
		align-items: center;
		color: #aaa;
	}
	.clickable {
		cursor: pointer;
		display: inline-flex;
		align-items: center;
		gap: 0.5rem;
		color: var(--title-color);
		text-decoration: none;
	}
	.clickable:hover {
		text-decoration: underline;
	}
	.clickable img {
		width: 1rem;
		height: 1rem;
		image-rendering: pixelated;
	}
</style>