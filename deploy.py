"""Backward-compatible wrapper for :mod:`james_library.bootstrap.deploy`."""

import james_library.bootstrap.deploy as _impl

globals().update(
    {name: getattr(_impl, name) for name in dir(_impl) if not name.startswith("__")}
)

if __name__ == "__main__":
    raise SystemExit(_impl.main())
