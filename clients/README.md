# Recommendation Engine Client SDKs

Official client libraries for the GrooveShop Recommendation Engine API.

## Available SDKs

### TypeScript/JavaScript Client ✅
**Location:** `clients/typescript/`
**Status:** Production Ready
**Version:** 1.0.0

- ✅ Full TypeScript support
- ✅ Modern ESM-only package (Node.js 22+)
- ✅ Uses native `fetch` API
- ✅ Works in Node.js and browsers
- ✅ Comprehensive test suite

**Installation:**
```bash
npm install @grooveshop/recommendation-engine-client
```

**Quick Start:**
```typescript
import { RecommendationClient } from '@grooveshop/recommendation-engine-client';

const client = new RecommendationClient({
  baseUrl: 'http://localhost:8080',
  apiKey: 'your-api-key',
});

const recs = await client.getUserRecommendations('user_123', {
  algorithm: 'hybrid',
  count: 10,
});
```

---

### Python Client ✅
**Location:** `clients/python/`
**Status:** Production Ready
**Version:** 1.0.0

- ✅ Full type hints (Python 3.14+)
- ✅ Async/await support with httpx
- ✅ Context manager support
- ✅ 85%+ test coverage
- ✅ Modern tooling (uv, ruff, mypy)

**Installation:**
```bash
pip install recommendation-engine-client
# or
uv add recommendation-engine-client
```

**Quick Start:**
```python
from recommendation_engine_client import RecommendationClient

async with RecommendationClient(
    base_url="http://localhost:8080",
    api_key="your-api-key"
) as client:
    recs = await client.get_user_recommendations(
        "user_123",
        {"algorithm": "hybrid", "count": 10}
    )
```

---

### Go Client 🚧
**Location:** `clients/go/` (planned)
**Status:** Pending Implementation

---

## SDK Feature Comparison

| Feature | TypeScript | Python | Go |
|---------|------------|--------|-----|
| Entity Management | ✅ | ✅ | 🚧 |
| Interaction Tracking | ✅ | ✅ | 🚧 |
| User Recommendations | ✅ | ✅ | 🚧 |
| Similar Entities | ✅ | ✅ | 🚧 |
| Trending Entities | ✅ | ✅ | 🚧 |
| Bulk Operations | ✅ | ✅ | 🚧 |
| Multi-tenancy | ✅ | ✅ | 🚧 |
| Type Safety | ✅ | ✅ | 🚧 |
| Async Support | ✅ | ✅ | 🚧 |
| Error Handling | ✅ | ✅ | 🚧 |
| Test Coverage | High | 85% | 🚧 |
| Documentation | ✅ | ✅ | 🚧 |
| Examples | ✅ | ✅ | 🚧 |

---

## Common API Operations

All SDKs support these core operations:

### Entity Operations
- Create entity
- Get entity by ID
- Update entity
- Delete entity
- Bulk import entities

### Interaction Operations
- Record interaction
- Get user interactions
- Bulk import interactions

### Recommendation Operations
- Get user recommendations (collaborative/content-based/hybrid)
- Get similar entities
- Get trending entities

### Health Checks
- Health check (liveness)
- Readiness check (dependencies)

---

## SDK Development Guidelines

### Requirements for All SDKs

1. **Type Safety:** Full type definitions/hints
2. **Error Handling:** Custom exceptions/errors
3. **Testing:** ≥80% code coverage
4. **Documentation:** Complete README with examples
5. **Modern Tooling:** Use latest stable tools
6. **Async Support:** Native async/await where applicable
7. **Multi-tenancy:** Support tenant_id parameter

### Code Quality Standards

- **Linting:** Zero linter errors
- **Formatting:** Consistent code style
- **Type Checking:** All public APIs fully typed
- **Tests:** Unit + integration tests
- **Examples:** At least 2 working examples

### Release Checklist

- [ ] All tests passing
- [ ] Code coverage ≥80%
- [ ] Documentation complete
- [ ] Examples working
- [ ] CHANGELOG updated
- [ ] Version bumped
- [ ] Build successful
- [ ] Published to package registry

---

## Contributing

Contributions are welcome! To add a new SDK or improve existing ones:

1. Follow the SDK development guidelines above
2. Ensure all quality checks pass
3. Add comprehensive documentation
4. Include working examples
5. Submit a pull request

---

## Support

For issues with specific SDKs:
- **TypeScript:** See `clients/typescript/README.md`
- **Python:** See `clients/python/README.md`
- **General:** https://github.com/grooveshop/recommendation-engine/issues

---

## License

All client SDKs are licensed under the MIT License.
