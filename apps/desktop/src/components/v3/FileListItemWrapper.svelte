<!-- This is a V3 replacement for `FileListItemWrapper.svelte` -->
<script lang="ts">
	import FileContextMenu from '$components/v3/FileContextMenu.svelte';
	import { BranchStack } from '$lib/branches/branch';
	import { draggableChips } from '$lib/dragging/draggable';
	import { ChangeDropData } from '$lib/dragging/draggables';
	import { getFilename } from '$lib/files/utils';
	import { ChangeSelectionService } from '$lib/selection/changeSelection.svelte';
	import { IdSelection } from '$lib/selection/idSelection.svelte';
	import { key, type SelectionParameters } from '$lib/selection/key';
	import { computeChangeStatus } from '$lib/utils/fileStatus';
	import { getContext, maybeGetContextStore } from '@gitbutler/shared/context';
	import FileListItemV3 from '@gitbutler/ui/file/FileListItemV3.svelte';
	import type { TreeChange } from '$lib/hunks/change';

	interface Props {
		containerFocused?: boolean;
		index: number;
		projectId: string;
		change: TreeChange;
		selectedFile: SelectionParameters;
		selected?: boolean;
		showCheckbox?: boolean;
		sticky?: boolean;
		onclick?: (e: MouseEvent) => void;
		onkeydown?: (e: KeyboardEvent) => void;
	}

	const {
		containerFocused,
		index,
		change,
		selectedFile,
		projectId,
		selected,
		showCheckbox,
		sticky,
		onclick,
		onkeydown
	}: Props = $props();

	const stack = maybeGetContextStore(BranchStack);
	const stackId = $derived($stack?.id);
	const idSelection = getContext(IdSelection);
	const changeSelection = getContext(ChangeSelectionService);

	let contextMenu = $state<ReturnType<typeof FileContextMenu>>();
	let draggableEl: HTMLDivElement | undefined = $state();

	const selection = $derived(changeSelection.getById(change.path));
	const indeterminate = $derived(selection.current && selection.current.type === 'partial');
	const selectedChanges = $derived(idSelection.treeChanges(projectId));

	function onCheck() {
		if (selection.current) {
			changeSelection.remove(change.path);
		} else {
			const { path, pathBytes } = change;
			changeSelection.add({
				type: 'full',
				path,
				pathBytes
			});
		}
	}

	function onContextMenu(e: MouseEvent) {
		if (selectedChanges.current.isSuccess && idSelection.has(change.path, selectedFile)) {
			const changes: TreeChange[] = selectedChanges.current.data;
			contextMenu?.open(e, { changes });
			return;
		}

		contextMenu?.open(e, { changes: [change] });
	}
</script>

<div
	bind:this={draggableEl}
	class:sticky
	use:draggableChips={{
		label: getFilename(change.path),
		filePath: change.path,
		data: new ChangeDropData(stackId || '', change, idSelection, selectedFile),
		viewportId: 'board-viewport',
		selector: '.selected-draggable'
	}}
>
	<FileContextMenu
		bind:this={contextMenu}
		trigger={draggableEl}
		isUnapplied={false}
		branchId={$stack?.id}
		isBinary={false}
	/>

	<FileListItemV3
		id={key({ ...selectedFile, path: change.path })}
		filePath={change.path}
		fileStatus={computeChangeStatus(change)}
		{selected}
		focused={idSelection.lastAddedIndex === index && containerFocused}
		{showCheckbox}
		checked={!!selection.current}
		{indeterminate}
		draggable={true}
		{onkeydown}
		locked={false}
		conflicted={false}
		onclick={(e) => {
			onclick?.(e);
		}}
		oncheck={onCheck}
		oncontextmenu={onContextMenu}
	/>
</div>

<style lang="postcss">
	.sticky {
		position: sticky;
		top: 0;
		z-index: var(--z-lifted);
		background-color: var(--clr-bg-1);
	}
</style>
