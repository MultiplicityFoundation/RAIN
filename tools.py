"""Backward-compatible wrapper for :mod:`james_library.utilities.tools`."""

import james_library.utilities.tools as _impl

globals().update(
    {name: getattr(_impl, name) for name in dir(_impl) if not name.startswith("__")}
)
