import { type BranchStack } from '$lib/branches/branch';
import { filesToSimpleOwnership } from '$lib/branches/ownership';
import { Commit, type DetailedCommit } from '$lib/commits/commit';
import { CommitDropData, FileDropData, HunkDropData } from '$lib/dragging/draggables';
import { RemoteFile } from '$lib/files/file';
import { LocalFile } from '$lib/files/file';
import type { BranchController } from '$lib/branches/branchController';
import type { Project } from '$lib/project/project';

export class CommitDragActions {
	constructor(
		private branchController: BranchController,
		private project: Project,
		private stack: BranchStack,
		private commit: DetailedCommit | Commit
	) {}

	acceptsAmend(dropData: unknown): boolean {
		if (this.commit instanceof Commit) {
			return false;
		}

		if (!this.project.ok_with_force_push && this.commit.isRemote) {
			return false;
		}

		if (this.commit.isIntegrated) {
			return false;
		}

		if (
			dropData instanceof HunkDropData &&
			dropData.branchId === this.stack.id &&
			dropData.commitId !== this.commit.id &&
			!this.commit.conflicted
		) {
			return true;
		} else if (
			dropData instanceof FileDropData &&
			dropData.branchId === this.stack.id &&
			dropData.commit?.id !== this.commit.id &&
			!this.commit.conflicted
		) {
			return true;
		} else {
			return false;
		}
	}

	onAmend(dropData: unknown): void {
		if (dropData instanceof HunkDropData) {
			// const newOwnership = `${dropData.hunk.filePath}:${dropData.hunk.id}`;
			this.branchController.amendBranch(this.stack.id, this.commit.id, [
				{
					previousPathBytes: null, // We dont have this, right?
					pathBytes: dropData.hunk.filePath, // Can we get the path in bytes here?
					hunkHeaders: [
						{
							oldStart: dropData.hunk.oldStart,
							oldLines: dropData.hunk.oldLines,
							newStart: dropData.hunk.newStart,
							newLines: dropData.hunk.newLines
						}
					]
				}
			]);
		} else if (dropData instanceof FileDropData) {
			if (dropData.file instanceof LocalFile) {
				const worktreeChanges = dropData.files.map((file) => {
					return {
						previousPathBytes: null,
						pathBytes: file.path, // Can we get the path in bytes here?
						hunkHeaders: [] // An empty list of hunk headers means use everything for the file
					};
				});
				this.branchController.amendBranch(this.stack.id, this.commit.id, worktreeChanges);
			} else if (dropData.file instanceof RemoteFile) {
				// this is a file from a commit, rather than an uncommitted file
				const newOwnership = filesToSimpleOwnership(dropData.files);
				if (dropData.commit) {
					this.branchController.moveCommitFile(
						this.stack.id,
						dropData.commit.id,
						this.commit.id,
						newOwnership
					);
				}
			}
		}
	}

	acceptsSquash(dropData: unknown): boolean {
		if (this.commit instanceof Commit) {
			return false;
		}
		if (!(dropData instanceof CommitDropData)) return false;
		if (dropData.branchId !== this.stack.id) return false;

		if (this.commit.conflicted || dropData.commit.conflicted) return false;
		if (this.commit.id === dropData.commit.id) return false;

		return true;
	}

	onSquash(dropData: unknown): void {
		if (this.commit instanceof Commit) {
			return;
		}
		if (dropData instanceof CommitDropData) {
			this.branchController.squashBranchCommit(
				dropData.branchId,
				dropData.commit.id,
				this.commit.id
			);
		}
	}
}

export class CommitDragActionsFactory {
	constructor(
		private branchController: BranchController,
		private project: Project
	) {}

	build(stack: BranchStack, commit: DetailedCommit | Commit) {
		return new CommitDragActions(this.branchController, this.project, stack, commit);
	}
}
