"""Custom exceptions for the Recommendation Engine client."""

from typing import Any


class RecommendationError(Exception):
    """Base exception for Recommendation Engine API errors."""

    def __init__(self, message: str, code: int = 0, details: Any | None = None) -> None:
        """Initialize the error.

        Args:
            message: Error message
            code: HTTP status code or error code
            details: Additional error details
        """
        super().__init__(message)
        self.message = message
        self.code = code
        self.details = details

    def __str__(self) -> str:
        """Return string representation of the error."""
        if self.code:
            return f"[{self.code}] {self.message}"
        return self.message

    def __repr__(self) -> str:
        """Return detailed representation of the error."""
        return (
            f"{self.__class__.__name__}("
            f"message={self.message!r}, "
            f"code={self.code!r}, "
            f"details={self.details!r})"
        )


class TimeoutError(RecommendationError):
    """Request timeout error."""

    def __init__(self, timeout: float) -> None:
        """Initialize timeout error.

        Args:
            timeout: Timeout duration in seconds
        """
        super().__init__(f"Request timeout after {timeout}s", code=408)


class NetworkError(RecommendationError):
    """Network connection error."""

    def __init__(self, message: str) -> None:
        """Initialize network error.

        Args:
            message: Error message
        """
        super().__init__(message, code=0)
