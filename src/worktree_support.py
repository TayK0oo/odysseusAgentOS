"""
Git Worktree support for Odysseus.

Enables forking projects into isolated worktrees for exploration,
then merging back or discarding. SFD UC-06, UC-07.
"""

import logging, os, subprocess
from typing import Optional

logger = logging.getLogger(__name__)

def worktree_create(project_path: str, branch_name: str) -> Optional[str]:
    """Create a git worktree for a project branch."""
    try:
        worktree_dir = f"{project_path}-{branch_name}"
        subprocess.run(
            ["git", "worktree", "add", worktree_dir, "-b", branch_name],
            cwd=project_path, capture_output=True, check=True, text=True
        )
        logger.info("Worktree created: %s", worktree_dir)
        return worktree_dir
    except subprocess.CalledProcessError as e:
        logger.error("Worktree create failed: %s", e.stderr)
        return None

def worktree_list(project_path: str) -> list[str]:
    """List all worktrees for a project."""
    try:
        r = subprocess.run(
            ["git", "worktree", "list", "--porcelain"],
            cwd=project_path, capture_output=True, text=True
        )
        worktrees = []
        for line in r.stdout.split("\n"):
            if line.startswith("worktree "):
                worktrees.append(line.replace("worktree ", "").strip())
        return worktrees
    except Exception:
        return []

def worktree_merge(worktree_path: str, project_path: str) -> bool:
    """Merge a worktree back into the main project."""
    try:
        branch = os.path.basename(worktree_path).split("-")[-1]
        subprocess.run(
            ["git", "checkout", "main" if branch == "main" else "dev"],
            cwd=project_path, capture_output=True, check=True
        )
        subprocess.run(
            ["git", "merge", branch],
            cwd=project_path, capture_output=True, check=True
        )
        subprocess.run(
            ["git", "worktree", "remove", worktree_path],
            cwd=project_path, capture_output=True, check=True
        )
        logger.info("Worktree merged and removed: %s", worktree_path)
        return True
    except subprocess.CalledProcessError as e:
        logger.error("Worktree merge failed: %s", e.stderr)
        return False

def worktree_discard(worktree_path: str, project_path: str) -> bool:
    """Discard a worktree without merging."""
    try:
        subprocess.run(
            ["git", "worktree", "remove", "--force", worktree_path],
            cwd=project_path, capture_output=True, check=True
        )
        logger.info("Worktree discarded: %s", worktree_path)
        return True
    except subprocess.CalledProcessError as e:
        logger.error("Worktree discard failed: %s", e.stderr)
        return False
