"""Backward-compatible wrapper for :mod:`james_library.bootstrap.rain_preflight_check`."""

import james_library.bootstrap.rain_preflight_check as _impl

globals().update(
    {name: getattr(_impl, name) for name in dir(_impl) if not name.startswith("__")}
)
