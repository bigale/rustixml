#!/home/bigale/repos/gitforai-integrations/.venv/bin/python3
"""
GitForAI Context Injection Hook for Claude Code

This hook automatically injects relevant Git commit history context
into every user query using semantic search over the repository's
commit database.

Hook Type: UserPromptSubmit
Pattern: Automatic context injection (Blueprint Pattern 1)
"""

import sys
import os
from pathlib import Path

# Add gitforai-integrations to Python path so we can import the hook modules
gitforai_root = Path("/home/bigale/repos/gitforai-integrations")
sys.path.insert(0, str(gitforai_root))

from adapters.claude_code.hooks.context_injector import GitContextHook
from adapters.claude_code.hooks.config import HookConfig

# Initialize the hook with configuration
# Environment variables can override these defaults
config = HookConfig(
    db_path=Path(os.getenv("GITFORAI_DB_PATH", "/home/bigale/repos/rustixml/.gitforai/vectordb")),
    repo_path=Path(os.getenv("GITFORAI_REPO_PATH", "/home/bigale/repos/rustixml")),
    max_results=int(os.getenv("GITFORAI_MAX_RESULTS", "5")),
    similarity_threshold=float(os.getenv("GITFORAI_SIMILARITY_THRESHOLD", "0.7")),
    enable_delta_analysis=os.getenv("GITFORAI_ENABLE_DELTA", "true").lower() == "true",
    enable_pattern_detection=os.getenv("GITFORAI_ENABLE_PATTERNS", "true").lower() == "true",
)

# Global hook instance (initialized once per session)
_hook = None


def get_hook():
    """Get or create the global hook instance."""
    global _hook
    if _hook is None:
        _hook = GitContextHook(config=config)
    return _hook


async def handle(event: dict) -> dict:
    """
    Hook handler called by Claude Code on UserPromptSubmit event.

    Args:
        event: Event data from Claude Code
            {
                "userPrompt": str,  # The user's message
                "context": {
                    "cwd": str,  # Current working directory
                    "conversationId": str,  # Conversation identifier
                    "messageId": str,  # Message identifier
                    ...
                }
            }

    Returns:
        dict: Hook result
            {
                "additionalSystemPrompt": str,  # Git context to inject
                # or empty dict if no context to inject
            }
    """
    try:
        hook = get_hook()
        result = await hook.on_user_prompt_submit(event)
        return result
    except Exception as e:
        # Log error but don't break Claude Code
        print(f"[GitForAI Hook Error] {e}", file=sys.stderr)
        return {}


# For synchronous hook systems (if needed)
def handle_sync(event: dict) -> dict:
    """Synchronous wrapper for the async handler."""
    import asyncio
    try:
        loop = asyncio.get_event_loop()
    except RuntimeError:
        loop = asyncio.new_event_loop()
        asyncio.set_event_loop(loop)

    return loop.run_until_complete(handle(event))
