# Changelog

All notable changes to the Recommendation Engine TypeScript Client will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.0] - 2025-10-22

### Added
- Initial release of the TypeScript/JavaScript client library
- Full TypeScript support with comprehensive type definitions
- Support for all Recommendation Engine API endpoints:
  - Entity operations (create, read, update, delete, bulk import)
  - Interaction operations (create, list, bulk import)
  - Recommendation operations (user recommendations, similar entities, trending)
  - Health check endpoints
- Automatic error handling with `RecommendationError` class
- Multi-tenancy support
- API key authentication support
- Configurable timeout and custom headers
- Works in both Node.js and browser environments
- Comprehensive documentation and examples
- ESLint and Prettier configuration
- Full JSDoc comments for IDE autocomplete

### Features
- Promise-based async API
- Automatic error wrapping and handling
- Type-safe request and response objects
- Support for custom HTTP headers
- Batch operations for bulk data import
- Flexible query parameters for all endpoints

[1.0.0]: https://github.com/grooveshop/recommendation-engine/releases/tag/typescript-v1.0.0
