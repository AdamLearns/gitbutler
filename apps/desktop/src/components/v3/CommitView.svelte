<script lang="ts">
	import ConfigurableScrollableContainer from '$components/ConfigurableScrollableContainer.svelte';
	import ReduxResult from '$components/ReduxResult.svelte';
	import ChangedFiles from '$components/v3/ChangedFiles.svelte';
	import CommitDetails from '$components/v3/CommitDetails.svelte';
	import CommitHeader from '$components/v3/CommitHeader.svelte';
	import Drawer from '$components/v3/Drawer.svelte';
	import { StackService } from '$lib/stacks/stackService.svelte';
	import { inject } from '@gitbutler/shared/context';
	import type { CommitKey } from '$lib/commits/commit';

	type Props = {
		projectId: string;
		stackId: string;
		commitKey: CommitKey;
		onclick?: () => void;
	};

	const { projectId, stackId, commitKey, onclick }: Props = $props();

	const [stackService] = inject(StackService);
	const commitResult = $derived(
		commitKey.upstream
			? stackService.upstreamCommitById(projectId, commitKey)
			: stackService.commitById(projectId, commitKey)
	);
</script>

<ReduxResult result={commitResult.current}>
	{#snippet children(commit)}
		<Drawer {projectId} {stackId}>
			{#snippet header()}
				<CommitHeader {commit} />
			{/snippet}
			<ConfigurableScrollableContainer>
				<div class="commit-view">
					<CommitDetails {projectId} {commit} {onclick} />
					<ChangedFiles type="commit" {projectId} commitId={commitKey.commitId} />
				</div>
			</ConfigurableScrollableContainer>
		</Drawer>
	{/snippet}
</ReduxResult>

<style>
	.commit-view {
		position: relative;
		min-height: 100%;
		flex: 1;
		display: flex;
		flex-direction: column;
		gap: 14px;
	}
</style>
