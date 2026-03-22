"""Backward-compatible wrapper for :mod:`james_library.launcher.rain_lab`."""

import james_library.launcher.rain_lab as _impl

globals().update(
    {name: getattr(_impl, name) for name in dir(_impl) if not name.startswith("__")}
)

if __name__ == "__main__":
    raise SystemExit(_impl.main())
