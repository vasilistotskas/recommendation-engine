# Recommendation Engine Helm Chart

This Helm chart deploys the Recommendation Engine microservice to Kubernetes.

## Prerequisites

- Kubernetes 1.19+
- Helm 3.0+
- PostgreSQL with pgvector extension
- Redis

## Installing the Chart

### Development Environment

```bash
helm install recommendation-engine ./helm/recommendation-engine \
  -f ./helm/recommendation-engine/values-dev.yaml \
  --set secrets.databaseUrl="postgresql://postgres:password@postgres:5432/recommendations" \
  --set secrets.redisUrl="redis://redis:6379" \
  --set secrets.apiKey="dev-api-key"
```

### Production Environment

```bash
helm install recommendation-engine ./helm/recommendation-engine \
  -f ./helm/recommendation-engine/values-prod.yaml \
  --set secrets.databaseUrl="postgresql://user:pass@prod-db:5432/recommendations?sslmode=require" \
  --set secrets.redisUrl="redis://prod-redis:6379" \
  --set secrets.apiKey="$(cat /path/to/prod-api-key)"
```

## Upgrading the Chart

```bash
helm upgrade recommendation-engine ./helm/recommendation-engine \
  -f ./helm/recommendation-engine/values-prod.yaml
```

## Uninstalling the Chart

```bash
helm uninstall recommendation-engine
```

## Configuration

The following table lists the configurable parameters of the chart and their default values.

| Parameter | Description | Default |
|-----------|-------------|---------|
| `replicaCount` | Number of replicas | `3` |
| `image.repository` | Image repository | `grooveshop/recommendation-engine` |
| `image.tag` | Image tag | `latest` |
| `image.pullPolicy` | Image pull policy | `Always` |
| `service.type` | Service type | `ClusterIP` |
| `service.port` | Service port | `80` |
| `ingress.enabled` | Enable ingress | `true` |
| `ingress.className` | Ingress class name | `nginx` |
| `ingress.hosts` | Ingress hosts | See values.yaml |
| `resources.requests.memory` | Memory request | `512Mi` |
| `resources.requests.cpu` | CPU request | `250m` |
| `resources.limits.memory` | Memory limit | `2Gi` |
| `resources.limits.cpu` | CPU limit | `1000m` |
| `autoscaling.enabled` | Enable HPA | `true` |
| `autoscaling.minReplicas` | Minimum replicas | `3` |
| `autoscaling.maxReplicas` | Maximum replicas | `20` |
| `config.logLevel` | Log level | `info` |
| `config.collaborativeWeight` | Collaborative algorithm weight | `0.6` |
| `config.contentWeight` | Content-based algorithm weight | `0.4` |
| `secrets.databaseUrl` | PostgreSQL connection string | Required |
| `secrets.redisUrl` | Redis connection string | Required |
| `secrets.apiKey` | API authentication key | Required |

For a complete list of parameters, see `values.yaml`.

## Using with External Secrets

For production deployments, it's recommended to use an external secret management system like:

- Kubernetes External Secrets Operator
- Sealed Secrets
- HashiCorp Vault
- AWS Secrets Manager
- Google Secret Manager

Example with External Secrets Operator:

```yaml
apiVersion: external-secrets.io/v1beta1
kind: ExternalSecret
metadata:
  name: recommendation-engine-external-secret
spec:
  refreshInterval: 1h
  secretStoreRef:
    name: aws-secrets-manager
    kind: SecretStore
  target:
    name: recommendation-engine
    creationPolicy: Owner
  data:
    - secretKey: database_url
      remoteRef:
        key: prod/recommendation-engine/database-url
    - secretKey: redis_url
      remoteRef:
        key: prod/recommendation-engine/redis-url
    - secretKey: api_key
      remoteRef:
        key: prod/recommendation-engine/api-key
```

## Monitoring

The chart exposes Prometheus metrics at `/metrics`. Configure your Prometheus instance to scrape this endpoint:

```yaml
apiVersion: v1
kind: ServiceMonitor
metadata:
  name: recommendation-engine
spec:
  selector:
    matchLabels:
      app.kubernetes.io/name: recommendation-engine
  endpoints:
  - port: http
    path: /metrics
    interval: 30s
```

## Health Checks

The chart configures liveness and readiness probes:

- Liveness: `GET /health`
- Readiness: `GET /ready`

## Support

For issues and questions, please open an issue at: https://github.com/grooveshop/recommendation-engine/issues
