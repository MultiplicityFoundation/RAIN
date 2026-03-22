"""Backward-compatible wrapper for :mod:`james_library.launcher.meeting_workflow`."""

import james_library.launcher.meeting_workflow as _impl

globals().update(
    {name: getattr(_impl, name) for name in dir(_impl) if not name.startswith("__")}
)
