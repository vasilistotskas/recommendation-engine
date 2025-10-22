"""Validation script for the recommendation-engine-client package."""

import sys
from pathlib import Path


def validate_imports() -> bool:
    """Validate all package imports work correctly."""
    print("Validating imports...")
    try:
        from recommendation_engine_client import (
            Algorithm,
            ApiError,
            AttributeValue,
            Attributes,
            BulkEntityItem,
            BulkImportEntitiesRequest,
            BulkImportError,
            BulkImportInteractionsRequest,
            BulkImportResponse,
            BulkInteractionItem,
            CreateEntityRequest,
            CreateInteractionRequest,
            Entity,
            EntityRecommendationsQuery,
            ErrorResponse,
            Interaction,
            InteractionType,
            NetworkError,
            RecommendationClient,
            RecommendationClientConfig,
            RecommendationError,
            RecommendationResponse,
            ScoredEntity,
            TimeoutError,
            TrendingEntitiesQuery,
            TrendingEntitiesResponse,
            UpdateEntityRequest,
            UserRecommendationsQuery,
            __version__,
        )

        print(f"  [OK] All imports successful (version {__version__})")
        return True
    except ImportError as e:
        print(f"  [FAIL] Import error: {e}")
        return False


def validate_client_instantiation() -> bool:
    """Validate client can be instantiated."""
    print("Validating client instantiation...")
    try:
        from recommendation_engine_client import RecommendationClient

        # Test basic instantiation
        client = RecommendationClient(base_url="http://test.example.com")

        # Test with all options
        client_full = RecommendationClient(
            base_url="http://test.example.com",
            api_key="test-key",
            timeout=60.0,
            headers={"X-Test": "value"},
        )

        print("  [OK] Client instantiation successful")
        return True
    except Exception as e:
        print(f"  [FAIL] Instantiation error: {e}")
        return False


def validate_type_annotations() -> bool:
    """Validate type annotations are present."""
    print("Validating type annotations...")
    try:
        from recommendation_engine_client.client import RecommendationClient
        from recommendation_engine_client.types import Entity, Interaction

        # Check that type hints are available
        if not hasattr(RecommendationClient.__init__, "__annotations__"):
            print("  [FAIL] No type annotations found")
            return False

        print("  [OK] Type annotations present")
        return True
    except Exception as e:
        print(f"  [FAIL] Type annotation check error: {e}")
        return False


def validate_file_structure() -> bool:
    """Validate package file structure."""
    print("Validating file structure...")

    base_dir = Path(__file__).parent
    required_files = [
        "src/recommendation_engine_client/__init__.py",
        "src/recommendation_engine_client/client.py",
        "src/recommendation_engine_client/exceptions.py",
        "src/recommendation_engine_client/types.py",
        "tests/test_client.py",
        "examples/basic_usage.py",
        "examples/e_commerce.py",
        "README.md",
        "LICENSE",
        "pyproject.toml",
        "CHANGELOG.md",
    ]

    missing_files = []
    for file_path in required_files:
        if not (base_dir / file_path).exists():
            missing_files.append(file_path)

    if missing_files:
        print(f"  [FAIL] Missing files: {', '.join(missing_files)}")
        return False

    print("  [OK] All required files present")
    return True


def validate_package_metadata() -> bool:
    """Validate package metadata."""
    print("Validating package metadata...")
    try:
        import tomllib

        base_dir = Path(__file__).parent
        with open(base_dir / "pyproject.toml", "rb") as f:
            config = tomllib.load(f)

        project = config.get("project", {})

        # Check required fields
        required_fields = ["name", "version", "description", "requires-python", "dependencies"]
        missing_fields = [field for field in required_fields if field not in project]

        if missing_fields:
            print(f"  [FAIL] Missing metadata fields: {', '.join(missing_fields)}")
            return False

        # Validate specific values
        if project["name"] != "recommendation-engine-client":
            print(f"  [FAIL] Wrong package name: {project['name']}")
            return False

        if project["version"] != "1.0.0":
            print(f"  [FAIL] Wrong version: {project['version']}")
            return False

        print("  [OK] Package metadata valid")
        return True
    except Exception as e:
        print(f"  [FAIL] Metadata validation error: {e}")
        return False


def main() -> int:
    """Run all validation checks."""
    print("=" * 60)
    print("Recommendation Engine Python Client - Validation")
    print("=" * 60)
    print()

    checks = [
        validate_file_structure,
        validate_package_metadata,
        validate_imports,
        validate_client_instantiation,
        validate_type_annotations,
    ]

    results = []
    for check in checks:
        result = check()
        results.append(result)
        print()

    print("=" * 60)
    passed = sum(results)
    total = len(results)

    if passed == total:
        print(f"SUCCESS: All {total} validation checks passed!")
        print("=" * 60)
        return 0
    else:
        print(f"FAILURE: {passed}/{total} validation checks passed")
        print("=" * 60)
        return 1


if __name__ == "__main__":
    sys.exit(main())
