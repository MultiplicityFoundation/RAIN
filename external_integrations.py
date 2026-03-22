"""Backward-compatible wrapper for :mod:`james_library.services.external_integrations`."""

import james_library.services.external_integrations as _impl

globals().update(
    {name: getattr(_impl, name) for name in dir(_impl) if not name.startswith("__")}
)
