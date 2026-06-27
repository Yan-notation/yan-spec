"""YAN (Yet Another Notation) parser for Python."""

from .parser import YANParser, YANParseError

__version__ = "1.0.0"
__all__ = ["YANParser", "YANParseError", "parse", "stringify"]


def parse(source: str) -> dict:
    """Parse a YAN source string into a Python dict."""
    return YANParser().parse(source)


def stringify(obj: dict, **kwargs) -> str:
    """Convert a Python dict to a YAN string."""
    return YANParser().stringify(obj, **kwargs)
