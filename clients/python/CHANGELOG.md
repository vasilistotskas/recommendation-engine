# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.0] - 2025-10-22

### Added
- Initial release of the Recommendation Engine Python client
- Full async/await support with httpx
- Complete type hints for Python 3.14+
- Support for all API endpoints:
  - Entity management (create, read, update, delete)
  - Interaction tracking
  - User recommendations (collaborative, content-based, hybrid)
  - Similar entity recommendations
  - Trending entities
  - Health and readiness checks
- Multi-tenancy support
- Bulk import operations for entities and interactions
- Context manager support for automatic resource cleanup
- Custom exceptions for better error handling
- Comprehensive test suite with 85%+ coverage
- Full documentation and examples
- Modern tooling: uv, ruff, mypy, pytest

### Documentation
- Complete README with usage examples
- API reference documentation
- Two example files:
  - `basic_usage.py` - Basic client usage
  - `e_commerce.py` - E-commerce recommendation flow
