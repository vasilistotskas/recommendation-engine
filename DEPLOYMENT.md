# Deployment Guide

> Comprehensive deployment strategies for the Recommendation Engine

This guide covers production deployment scenarios including Kubernetes rolling updates, zero-downtime deployments, database migrations, monitoring setup, and scaling strategies.

---

## Table of Contents

- [Quick Start Deployment](#quick-start-deployment)
- [Kubernetes Deployment](#kubernetes-deployment)
- [Rolling Updates (Zero-Downtime)](#rolling-updates-zero-downtime)
- [Database Migration Strategies](#database-migration-strategies)
- [Monitoring and Observability](#monitoring-and-observability)
- [Scaling Strategies](#scaling-strategies)
- [High Availability Setup](#high-availability-setup)
- [Disaster Recovery](#disaster-recovery)
- [Performance Tuning](#performance-tuning)

---

## Quick Start Deployment

### Docker Compose (Development/Testing)

The fastest way to deploy locally:

```bash
# Clone repository
git clone https://github.com/vasilistotskas/recommendation-engine.git
cd recommendation-engine

# Start all services
docker-compose up -d

# Check logs
docker-compose logs -f recommendation-api

# Stop services
docker-compose down
```

**Services included:**
- PostgreSQL 17 with pgvector
- Redis 7
- Recommendation API

---

## Kubernetes Deployment

### Prerequisites

- Kubernetes cluster (1.25+)
- kubectl configured
- Helm 3.0+ (optional)
- PostgreSQL with pgvector (managed or self-hosted)
- Redis (managed or self-hosted)

### Step 1: Create Namespace

```bash
kubectl create namespace recommendation-engine
kubectl config set-context --current --namespace=recommendation-engine
```

### Step 2: Create Secrets

```bash
# Create secret for sensitive configuration
kubectl create secret generic recommendation-secrets \
  --from-literal=database-url='postgresql://user:password@postgres-host:5432/recommendations' \
  --from-literal=redis-url='redis://redis-host:6379' \
  --from-literal=api-key='your-secure-api-key-here'
```

### Step 3: Create ConfigMap

```yaml
# config/configmap.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: recommendation-config
  namespace: recommendation-engine
data:
  PORT: "8080"
  LOG_LEVEL: "info"
  RUST_LOG: "info,recommendation_api=debug"

  # Algorithm Configuration
  COLLABORATIVE_WEIGHT: "0.6"
  CONTENT_BASED_WEIGHT: "0.4"
  SIMILARITY_THRESHOLD: "0.5"
  DEFAULT_RECOMMENDATION_COUNT: "10"
  MAX_RECOMMENDATION_COUNT: "100"
  FEATURE_VECTOR_DIMENSION: "512"

  # Performance Configuration
  DATABASE_MAX_CONNECTIONS: "20"
  DATABASE_MIN_CONNECTIONS: "5"
  REDIS_POOL_SIZE: "10"
  WORKER_THREADS: "4"

  # Cache TTLs
  RECOMMENDATION_CACHE_TTL_SECS: "300"
  TRENDING_CACHE_TTL_SECS: "3600"
  USER_PREFERENCE_CACHE_TTL_SECS: "600"

  # Rate Limiting
  RATE_LIMIT_ENABLED: "true"
  RATE_LIMIT_REQUESTS_PER_MINUTE: "1000"

  # Model Updates
  INCREMENTAL_UPDATE_INTERVAL_SECS: "10"
  FULL_REBUILD_INTERVAL_HOURS: "24"

  # Graceful Shutdown
  SHUTDOWN_TIMEOUT_SECS: "30"
```

Apply the ConfigMap:
```bash
kubectl apply -f config/configmap.yaml
```

### Step 4: Create Deployment

```yaml
# deployments/recommendation-api.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: recommendation-api
  namespace: recommendation-engine
  labels:
    app: recommendation-api
    version: v1
spec:
  replicas: 3
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxSurge: 1
      maxUnavailable: 0  # Zero-downtime deployments
  selector:
    matchLabels:
      app: recommendation-api
  template:
    metadata:
      labels:
        app: recommendation-api
        version: v1
      annotations:
        prometheus.io/scrape: "true"
        prometheus.io/port: "8080"
        prometheus.io/path: "/metrics"
    spec:
      # Anti-affinity to spread pods across nodes
      affinity:
        podAntiAffinity:
          preferredDuringSchedulingIgnoredDuringExecution:
          - weight: 100
            podAffinityTerm:
              labelSelector:
                matchExpressions:
                - key: app
                  operator: In
                  values:
                  - recommendation-api
              topologyKey: kubernetes.io/hostname

      containers:
      - name: api
        image: ghcr.io/vasilistotskas/recommendation-engine:latest
        imagePullPolicy: Always
        ports:
        - name: http
          containerPort: 8080
          protocol: TCP

        # Environment variables from ConfigMap
        envFrom:
        - configMapRef:
            name: recommendation-config

        # Sensitive environment variables from Secret
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: recommendation-secrets
              key: database-url
        - name: REDIS_URL
          valueFrom:
            secretKeyRef:
              name: recommendation-secrets
              key: redis-url
        - name: API_KEY
          valueFrom:
            secretKeyRef:
              name: recommendation-secrets
              key: api-key

        # Liveness probe: Is the container alive?
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
            scheme: HTTP
          initialDelaySeconds: 15
          periodSeconds: 10
          timeoutSeconds: 3
          successThreshold: 1
          failureThreshold: 3

        # Readiness probe: Can the container accept traffic?
        readinessProbe:
          httpGet:
            path: /ready
            port: 8080
            scheme: HTTP
          initialDelaySeconds: 10
          periodSeconds: 5
          timeoutSeconds: 3
          successThreshold: 1
          failureThreshold: 2

        # Startup probe: For slow-starting containers
        startupProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 0
          periodSeconds: 5
          timeoutSeconds: 3
          failureThreshold: 30  # 30 * 5s = 150s max startup time

        # Resource requests and limits
        resources:
          requests:
            memory: "512Mi"
            cpu: "500m"
          limits:
            memory: "2Gi"
            cpu: "2000m"

        # Graceful shutdown configuration
        lifecycle:
          preStop:
            exec:
              command: ["/bin/sh", "-c", "sleep 10"]  # Give time for load balancer to deregister

        # Security context
        securityContext:
          runAsNonRoot: true
          runAsUser: 1000
          allowPrivilegeEscalation: false
          capabilities:
            drop:
            - ALL
          readOnlyRootFilesystem: true

        # Volume mounts for tmp directories
        volumeMounts:
        - name: tmp
          mountPath: /tmp
        - name: cache
          mountPath: /app/cache

      # Volumes
      volumes:
      - name: tmp
        emptyDir: {}
      - name: cache
        emptyDir: {}

      # Termination grace period (matches SHUTDOWN_TIMEOUT_SECS + buffer)
      terminationGracePeriodSeconds: 40
```

Apply the Deployment:
```bash
kubectl apply -f deployments/recommendation-api.yaml
```

### Step 5: Create Service

```yaml
# services/recommendation-api.yaml
apiVersion: v1
kind: Service
metadata:
  name: recommendation-api
  namespace: recommendation-engine
  labels:
    app: recommendation-api
spec:
  type: ClusterIP  # Use LoadBalancer or NodePort for external access
  selector:
    app: recommendation-api
  ports:
  - name: http
    port: 80
    targetPort: 8080
    protocol: TCP
  sessionAffinity: None
```

Apply the Service:
```bash
kubectl apply -f services/recommendation-api.yaml
```

### Step 6: Create Ingress (Optional)

```yaml
# ingress/recommendation-api.yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: recommendation-api
  namespace: recommendation-engine
  annotations:
    kubernetes.io/ingress.class: nginx
    cert-manager.io/cluster-issuer: letsencrypt-prod
    nginx.ingress.kubernetes.io/rate-limit: "100"
    nginx.ingress.kubernetes.io/ssl-redirect: "true"
spec:
  tls:
  - hosts:
    - api.recommendation-engine.example.com
    secretName: recommendation-api-tls
  rules:
  - host: api.recommendation-engine.example.com
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: recommendation-api
            port:
              number: 80
```

Apply the Ingress:
```bash
kubectl apply -f ingress/recommendation-api.yaml
```

---

## Rolling Updates (Zero-Downtime)

### How It Works

The Recommendation Engine is designed for zero-downtime deployments using Kubernetes rolling updates:

```
1. New pods are created alongside old pods
2. New pods start and pass readiness checks
3. Load balancer routes traffic to new pods
4. Old pods receive SIGTERM signal
5. Old pods mark themselves as "not ready" (readiness probe returns 503)
6. Load balancer stops routing new traffic to old pods
7. Old pods drain existing requests (up to 30 seconds)
8. Old pods shut down gracefully
9. Deployment completes
```

### Key Configuration

The following settings enable zero-downtime deployments:

**1. Deployment Strategy:**
```yaml
strategy:
  type: RollingUpdate
  rollingUpdate:
    maxSurge: 1           # Create 1 extra pod during update
    maxUnavailable: 0     # Never have fewer than desired replicas
```

**2. Readiness Probe:**
```yaml
readinessProbe:
  httpGet:
    path: /ready         # Returns 503 during shutdown
  failureThreshold: 2    # Mark unhealthy after 2 failed checks
```

**3. Graceful Shutdown:**
```yaml
env:
- name: SHUTDOWN_TIMEOUT_SECS
  value: "30"            # Drain requests for 30 seconds

terminationGracePeriodSeconds: 40  # Give 40s total (30s drain + 10s buffer)

lifecycle:
  preStop:
    exec:
      command: ["/bin/sh", "-c", "sleep 10"]  # Wait for LB to deregister
```

**4. Application Behavior:**
- On SIGTERM signal, the app:
  1. Sets readiness probe to return 503 (/ready endpoint)
  2. Stops accepting new HTTP connections
  3. Continues processing in-flight requests
  4. Waits up to `SHUTDOWN_TIMEOUT_SECS` for requests to complete
  5. Shuts down cleanly

### Performing a Rolling Update

```bash
# Update to new image version
kubectl set image deployment/recommendation-api \
  api=ghcr.io/vasilistotskas/recommendation-engine:v1.1.0 \
  --namespace=recommendation-engine

# Watch the rollout status
kubectl rollout status deployment/recommendation-api \
  --namespace=recommendation-engine

# Check pod status
kubectl get pods --namespace=recommendation-engine -w

# View rollout history
kubectl rollout history deployment/recommendation-api \
  --namespace=recommendation-engine
```

### Rollback Strategy

If issues are detected during rollout:

```bash
# Pause the rollout
kubectl rollout pause deployment/recommendation-api \
  --namespace=recommendation-engine

# Investigate issues
kubectl logs -l app=recommendation-api --tail=100 \
  --namespace=recommendation-engine

# Rollback to previous version
kubectl rollout undo deployment/recommendation-api \
  --namespace=recommendation-engine

# Rollback to specific revision
kubectl rollout undo deployment/recommendation-api \
  --to-revision=2 \
  --namespace=recommendation-engine

# Resume rollout (if paused)
kubectl rollout resume deployment/recommendation-api \
  --namespace=recommendation-engine
```

### Validation After Deployment

```bash
# 1. Check all pods are ready
kubectl get pods -l app=recommendation-api --namespace=recommendation-engine

# 2. Test health endpoints
POD_NAME=$(kubectl get pods -l app=recommendation-api -o jsonpath='{.items[0].metadata.name}' --namespace=recommendation-engine)
kubectl exec $POD_NAME --namespace=recommendation-engine -- curl -s http://localhost:8080/health
kubectl exec $POD_NAME --namespace=recommendation-engine -- curl -s http://localhost:8080/ready

# 3. Check metrics
kubectl exec $POD_NAME --namespace=recommendation-engine -- curl -s http://localhost:8080/metrics | grep http_requests_total

# 4. Monitor error rates
kubectl logs -l app=recommendation-api --tail=50 --namespace=recommendation-engine | grep ERROR
```

---

## Database Migration Strategies

### Strategy 1: Pre-Deployment Migration (Recommended)

Run migrations before deploying new application version:

```bash
# 1. Create a Kubernetes Job for migrations
```

```yaml
# jobs/migrate.yaml
apiVersion: batch/v1
kind: Job
metadata:
  name: recommendation-migrate-v1-1-0
  namespace: recommendation-engine
spec:
  template:
    spec:
      restartPolicy: OnFailure
      containers:
      - name: migrate
        image: ghcr.io/vasilistotskas/recommendation-engine:v1.1.0
        command: ["/bin/sh", "-c"]
        args:
        - |
          # Install sqlx-cli
          cargo install sqlx-cli --no-default-features --features postgres

          # Run migrations
          sqlx migrate run

          echo "Migration completed successfully"
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: recommendation-secrets
              key: database-url
```

```bash
# 2. Apply migration job
kubectl apply -f jobs/migrate.yaml

# 3. Wait for migration to complete
kubectl wait --for=condition=complete --timeout=300s job/recommendation-migrate-v1-1-0

# 4. Check migration logs
kubectl logs job/recommendation-migrate-v1-1-0

# 5. If successful, deploy new application version
kubectl apply -f deployments/recommendation-api.yaml

# 6. Clean up migration job
kubectl delete job recommendation-migrate-v1-1-0
```

### Strategy 2: Init Container Migration

Run migrations as init container (migrations run on every pod start):

```yaml
spec:
  template:
    spec:
      initContainers:
      - name: migrate
        image: ghcr.io/vasilistotskas/recommendation-engine:latest
        command: ["/bin/sh", "-c"]
        args:
        - |
          sqlx migrate run
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: recommendation-secrets
              key: database-url
```

**Pros:**
- Automatic migration on deployment
- No separate migration job needed

**Cons:**
- Multiple pods may try to run migrations simultaneously
- Slower pod startup time

### Strategy 3: Manual Migration (Production)

For production environments with strict change control:

```bash
# 1. Create database backup
kubectl exec -it postgres-pod -- pg_dump -U postgres recommendations > backup.sql

# 2. Apply migration in maintenance window
kubectl run -it --rm migrate \
  --image=ghcr.io/vasilistotskas/recommendation-engine:v1.1.0 \
  --restart=Never \
  --env=DATABASE_URL=$DATABASE_URL \
  -- sqlx migrate run

# 3. Verify migration
kubectl run -it --rm psql \
  --image=postgres:17 \
  --restart=Never \
  --env=PGPASSWORD=$DB_PASSWORD \
  -- psql -h $DB_HOST -U postgres -d recommendations -c "\dt"

# 4. Deploy new version
kubectl apply -f deployments/recommendation-api.yaml
```

### Backward Compatibility

**Golden Rule**: Migrations must be backward compatible with the previous application version.

**Example - Adding a Column:**

✅ **Good (Backward Compatible):**
```sql
-- Migration: Add new column with default value
ALTER TABLE entities
ADD COLUMN new_field TEXT DEFAULT '';

-- Old app version: Ignores new column
-- New app version: Uses new column
```

❌ **Bad (Breaking Change):**
```sql
-- Migration: Add required column without default
ALTER TABLE entities
ADD COLUMN required_field TEXT NOT NULL;

-- Old app version: FAILS (column doesn't exist)
```

**Multi-Step Migrations for Breaking Changes:**

```sql
-- Step 1 (Deploy with old app):
ALTER TABLE entities
ADD COLUMN new_name TEXT;

UPDATE entities SET new_name = old_name;

-- Step 2 (Deploy new app version):
-- App uses new_name column

-- Step 3 (After new app is stable):
ALTER TABLE entities
DROP COLUMN old_name;
```

---

## Monitoring and Observability

### Prometheus Setup

**1. Install Prometheus Operator:**
```bash
helm repo add prometheus-community https://prometheus-community.github.io/helm-charts
helm install prometheus prometheus-community/kube-prometheus-stack \
  --namespace monitoring \
  --create-namespace
```

**2. Create ServiceMonitor:**
```yaml
# monitoring/servicemonitor.yaml
apiVersion: monitoring.coreos.com/v1
kind: ServiceMonitor
metadata:
  name: recommendation-api
  namespace: recommendation-engine
  labels:
    app: recommendation-api
spec:
  selector:
    matchLabels:
      app: recommendation-api
  endpoints:
  - port: http
    path: /metrics
    interval: 30s
```

**3. Create Prometheus Rules:**
```yaml
# monitoring/prometheusrule.yaml
apiVersion: monitoring.coreos.com/v1
kind: PrometheusRule
metadata:
  name: recommendation-api-alerts
  namespace: recommendation-engine
spec:
  groups:
  - name: recommendation-api
    interval: 30s
    rules:
    # High error rate alert
    - alert: HighErrorRate
      expr: |
        rate(http_requests_errors_total[5m]) / rate(http_requests_total[5m]) > 0.05
      for: 5m
      labels:
        severity: warning
      annotations:
        summary: "High error rate detected"
        description: "Error rate is {{ $value | humanizePercentage }} (threshold: 5%)"

    # High latency alert
    - alert: HighLatency
      expr: |
        histogram_quantile(0.95, rate(http_request_duration_seconds_bucket[5m])) > 0.2
      for: 5m
      labels:
        severity: warning
      annotations:
        summary: "High request latency detected"
        description: "P95 latency is {{ $value }}s (threshold: 200ms)"

    # Low cache hit rate alert
    - alert: LowCacheHitRate
      expr: |
        redis_cache_hits_total / (redis_cache_hits_total + redis_cache_misses_total) < 0.70
      for: 10m
      labels:
        severity: info
      annotations:
        summary: "Low cache hit rate"
        description: "Cache hit rate is {{ $value | humanizePercentage }} (threshold: 70%)"

    # Database connection pool exhausted
    - alert: DatabasePoolExhausted
      expr: |
        database_pool_idle_connections == 0
      for: 2m
      labels:
        severity: critical
      annotations:
        summary: "Database connection pool exhausted"
        description: "No idle database connections available"
```

### Grafana Dashboards

**Import Pre-built Dashboard:**
```bash
# Dashboard ID: 14282 (Kubernetes cluster monitoring)
# Dashboard ID: 12900 (Prometheus exporters)
```

**Custom Recommendation Engine Dashboard:**
```json
{
  "dashboard": {
    "title": "Recommendation Engine",
    "panels": [
      {
        "title": "Request Rate",
        "targets": [
          {
            "expr": "rate(http_requests_total[5m])"
          }
        ]
      },
      {
        "title": "Error Rate",
        "targets": [
          {
            "expr": "rate(http_requests_errors_total[5m]) / rate(http_requests_total[5m])"
          }
        ]
      },
      {
        "title": "P95 Latency",
        "targets": [
          {
            "expr": "histogram_quantile(0.95, rate(http_request_duration_seconds_bucket[5m]))"
          }
        ]
      },
      {
        "title": "Cache Hit Rate",
        "targets": [
          {
            "expr": "redis_cache_hits_total / (redis_cache_hits_total + redis_cache_misses_total)"
          }
        ]
      }
    ]
  }
}
```

### Logging with ELK Stack

**1. Install Fluent Bit:**
```bash
helm repo add fluent https://fluent.github.io/helm-charts
helm install fluent-bit fluent/fluent-bit \
  --namespace logging \
  --create-namespace \
  --set backend.type=es \
  --set backend.es.host=elasticsearch \
  --set backend.es.port=9200
```

**2. Configure Structured Logging:**
```bash
# Add to ConfigMap
LOG_FORMAT: "json"
RUST_LOG: "info,recommendation_api=debug"
```

**3. Create Elasticsearch Index Pattern:**
```bash
# Access Kibana and create index pattern: fluent-bit-*
```

---

## Scaling Strategies

### Horizontal Pod Autoscaling (HPA)

**1. Create HPA based on CPU:**
```yaml
# autoscaling/hpa.yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: recommendation-api
  namespace: recommendation-engine
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: recommendation-api
  minReplicas: 3
  maxReplicas: 20
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
  - type: Resource
    resource:
      name: memory
      target:
        type: Utilization
        averageUtilization: 80
  behavior:
    scaleDown:
      stabilizationWindowSeconds: 300  # Wait 5min before scaling down
      policies:
      - type: Percent
        value: 50
        periodSeconds: 60
    scaleUp:
      stabilizationWindowSeconds: 0    # Scale up immediately
      policies:
      - type: Percent
        value: 100
        periodSeconds: 30
```

**2. Create HPA based on Custom Metrics:**
```yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: recommendation-api-custom
  namespace: recommendation-engine
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: recommendation-api
  minReplicas: 3
  maxReplicas: 20
  metrics:
  - type: Pods
    pods:
      metric:
        name: http_requests_per_second
      target:
        type: AverageValue
        averageValue: "1000"  # Scale when > 1000 req/s per pod
```

### Vertical Scaling

**Adjust resource requests/limits:**
```yaml
resources:
  requests:
    memory: "1Gi"      # Increased from 512Mi
    cpu: "1000m"       # Increased from 500m
  limits:
    memory: "4Gi"      # Increased from 2Gi
    cpu: "4000m"       # Increased from 2000m
```

### Database Scaling

**PostgreSQL Read Replicas:**
```yaml
env:
- name: DATABASE_READ_URL
  value: "postgresql://user:pass@postgres-replica:5432/recommendations"
- name: DATABASE_WRITE_URL
  value: "postgresql://user:pass@postgres-primary:5432/recommendations"
```

**Redis Cluster:**
```yaml
env:
- name: REDIS_URL
  value: "redis://redis-cluster:6379"
- name: REDIS_CLUSTER_MODE
  value: "true"
```

---

## High Availability Setup

### Multi-Region Deployment

```
Region A (Primary)          Region B (DR)
┌─────────────────┐        ┌─────────────────┐
│  K8s Cluster A  │        │  K8s Cluster B  │
│  ┌───────────┐  │        │  ┌───────────┐  │
│  │  API (3)  │  │        │  │  API (2)  │  │
│  └───────────┘  │        │  └───────────┘  │
│  ┌───────────┐  │        │  ┌───────────┐  │
│  │ PostgreSQL│◄─┼────────┼─▶│ PostgreSQL│  │
│  │ (Primary) │  │  Repl  │  │ (Replica) │  │
│  └───────────┘  │        │  └───────────┘  │
│  ┌───────────┐  │        │  ┌───────────┐  │
│  │   Redis   │◄─┼────────┼─▶│   Redis   │  │
│  │ (Primary) │  │  Repl  │  │ (Replica) │  │
│  └───────────┘  │        │  └───────────┘  │
└─────────────────┘        └─────────────────┘
        │                          │
        └──────────┬───────────────┘
                   ▼
          ┌─────────────────┐
          │  Global LB      │
          │  (CloudFlare/   │
          │   AWS Route53)  │
          └─────────────────┘
```

### Pod Disruption Budget

```yaml
# pdb/recommendation-api.yaml
apiVersion: policy/v1
kind: PodDisruptionBudget
metadata:
  name: recommendation-api
  namespace: recommendation-engine
spec:
  minAvailable: 2  # Always keep at least 2 pods running
  selector:
    matchLabels:
      app: recommendation-api
```

---

## Disaster Recovery

### Backup Strategy

**1. Database Backups:**
```bash
# Automated daily backups with pg_dump
kubectl create cronjob postgres-backup \
  --image=postgres:17 \
  --schedule="0 2 * * *" \
  -- pg_dump -h postgres -U postgres recommendations > /backup/db-$(date +%Y%m%d).sql
```

**2. Configuration Backups:**
```bash
# Backup all Kubernetes resources
kubectl get all,configmap,secret,ingress,pdb,hpa \
  --namespace=recommendation-engine \
  -o yaml > backup/k8s-resources-$(date +%Y%m%d).yaml
```

### Recovery Procedures

**Scenario 1: Pod Failure**
- Automatic: Kubernetes restarts failed pods
- No action needed (self-healing)

**Scenario 2: Node Failure**
- Automatic: Pods rescheduled to healthy nodes
- Monitor: `kubectl get pods -w`

**Scenario 3: Database Corruption**
```bash
# 1. Stop all API pods
kubectl scale deployment/recommendation-api --replicas=0

# 2. Restore from backup
kubectl exec -it postgres-pod -- psql -U postgres -d recommendations < backup.sql

# 3. Verify data
kubectl exec -it postgres-pod -- psql -U postgres -d recommendations -c "SELECT COUNT(*) FROM entities"

# 4. Restart API pods
kubectl scale deployment/recommendation-api --replicas=3
```

**Scenario 4: Complete Region Failure**
```bash
# 1. Update DNS to point to DR region
# 2. Promote PostgreSQL replica to primary
# 3. Update application configuration
# 4. Monitor metrics and logs
```

---

## Performance Tuning

### Application Tuning

```yaml
# Optimize for high throughput
env:
- name: WORKER_THREADS
  value: "8"  # Match CPU cores
- name: DATABASE_MAX_CONNECTIONS
  value: "50"  # Increase pool size
- name: REDIS_POOL_SIZE
  value: "20"
- name: RECOMMENDATION_CACHE_TTL_SECS
  value: "600"  # Cache longer
```

### Database Tuning

```sql
-- PostgreSQL configuration
ALTER SYSTEM SET shared_buffers = '4GB';
ALTER SYSTEM SET effective_cache_size = '12GB';
ALTER SYSTEM SET maintenance_work_mem = '1GB';
ALTER SYSTEM SET checkpoint_completion_target = 0.9;
ALTER SYSTEM SET wal_buffers = '16MB';
ALTER SYSTEM SET default_statistics_target = 100;
ALTER SYSTEM SET random_page_cost = 1.1;
ALTER SYSTEM SET effective_io_concurrency = 200;
ALTER SYSTEM SET max_worker_processes = 8;
ALTER SYSTEM SET max_parallel_workers_per_gather = 4;
ALTER SYSTEM SET max_parallel_workers = 8;

-- Reload configuration
SELECT pg_reload_conf();
```

### Network Optimization

```yaml
# Use NodePort for better performance
spec:
  type: NodePort
  externalTrafficPolicy: Local  # Preserve source IP, reduce hop
```

---

## Troubleshooting

### Common Issues

**Issue: Pods not starting**
```bash
# Check pod events
kubectl describe pod <pod-name>

# Check logs
kubectl logs <pod-name>

# Check resource availability
kubectl top nodes
kubectl describe node <node-name>
```

**Issue: Readiness probe failing**
```bash
# Test readiness endpoint
kubectl exec <pod-name> -- curl -v http://localhost:8080/ready

# Check database connectivity
kubectl exec <pod-name> -- env | grep DATABASE_URL

# Check Redis connectivity
kubectl exec <pod-name> -- env | grep REDIS_URL
```

**Issue: High memory usage**
```bash
# Check memory metrics
kubectl top pod <pod-name>

# Adjust cache TTLs (reduce memory footprint)
# Reduce RECOMMENDATION_CACHE_TTL_SECS
# Reduce REDIS_POOL_SIZE
```

---

## Security Checklist

- [ ] Secrets stored in Kubernetes Secrets (not ConfigMaps)
- [ ] TLS enabled on Ingress
- [ ] API key authentication enabled
- [ ] Rate limiting configured
- [ ] Non-root container user
- [ ] Read-only root filesystem
- [ ] Network policies configured
- [ ] Resource limits set
- [ ] Pod Security Standards enforced
- [ ] Regular security audits (`cargo audit`)
- [ ] Dependency updates automated (Dependabot)

---

## Performance Checklist

- [ ] HPA configured
- [ ] Pod Disruption Budget set
- [ ] Anti-affinity rules configured
- [ ] Readiness/liveness probes tuned
- [ ] Graceful shutdown configured
- [ ] Cache TTLs optimized
- [ ] Database connection pool sized correctly
- [ ] Monitoring and alerting enabled
- [ ] Load testing performed
- [ ] Resource requests/limits set appropriately

---

**For additional support, see:**
- [README.md](README.md) - General documentation
- [GitHub Issues](https://github.com/vasilistotskas/recommendation-engine/issues)
- [Performance Tuning Guide](PERFORMANCE_TUNING.md)
