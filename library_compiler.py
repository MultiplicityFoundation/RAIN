"""Backward-compatible wrapper for :mod:`james_library.utilities.library_compiler`."""

import james_library.utilities.library_compiler as _impl

globals().update(
    {name: getattr(_impl, name) for name in dir(_impl) if not name.startswith("__")}
)
