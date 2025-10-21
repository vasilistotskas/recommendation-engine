# ArgoCD Deployment

This directory contains ArgoCD Application manifests for deploying the Recommendation Engine using GitOps.

## Prerequisites

- ArgoCD installed in your Kubernetes cluster
- Access to the Git repository
- Appropriate RBAC permissions

## Deployment Environments

### Development Environment

```bash
kubectl apply -f argocd/application-dev.yaml
```

This will:
- Deploy from the `develop` branch
- Use `values-dev.yaml` configuration
- Deploy to `recommendation-engine-dev` namespace
- Enable auto-sync and self-healing

### Production Environment

```bash
kubectl apply -f argocd/application-prod.yaml
```

This will:
- Deploy from specific version tags (e.g., `v1.0.0`)
- Use `values-prod.yaml` configuration
- Deploy to `recommendation-engine` namespace
- **Require manual sync** for safety
- Send notifications on sync events

## Accessing ArgoCD UI

```bash
# Port-forward to access ArgoCD UI
kubectl port-forward svc/argocd-server -n argocd 8080:443

# Get admin password
kubectl -n argocd get secret argocd-initial-admin-secret -o jsonpath="{.data.password}" | base64 -d
```

Then navigate to: https://localhost:8080

## Syncing Applications

### Manual Sync via CLI

```bash
# Sync development
argocd app sync recommendation-engine-dev

# Sync production (requires approval)
argocd app sync recommendation-engine-prod
```

### View Application Status

```bash
# Check status
argocd app get recommendation-engine-prod

# View sync history
argocd app history recommendation-engine-prod

# View application logs
argocd app logs recommendation-engine-prod
```

## Rollback

To rollback to a previous version:

```bash
# List deployment history
argocd app history recommendation-engine-prod

# Rollback to specific revision
argocd app rollback recommendation-engine-prod <revision-number>
```

Or update the `targetRevision` in the Application manifest:

```yaml
spec:
  source:
    targetRevision: v0.9.0  # Previous version
```

Then apply:

```bash
kubectl apply -f argocd/application-prod.yaml
argocd app sync recommendation-engine-prod
```

## Auto-sync Configuration

### Development
- **Auto-sync**: Enabled
- **Self-heal**: Enabled
- **Prune**: Enabled

### Production
- **Auto-sync**: Disabled (manual approval required)
- **Self-heal**: Disabled
- **Prune**: Disabled

## Notifications

Production deployments send notifications to Slack on:
- Successful sync
- Failed sync
- Health status changes

Configure ArgoCD notifications by setting up the appropriate ConfigMap and Secret.

## Ignoring Differences

The Application manifests ignore differences in:
- `spec.replicas` for Deployments (managed by HPA)

This prevents ArgoCD from detecting drift when the HPA scales the deployment.

## Health Checks

ArgoCD automatically monitors:
- Deployment rollout status
- Pod health (liveness/readiness probes)
- Service availability

## Secrets Management

Secrets should be managed using one of these approaches:

1. **Sealed Secrets**
2. **External Secrets Operator**
3. **ArgoCD Vault Plugin**

Example with External Secrets Operator is documented in the Helm chart README.

## Troubleshooting

### Application stuck in sync

```bash
# Check sync status
argocd app get recommendation-engine-prod

# View detailed sync status
argocd app sync recommendation-engine-prod --dry-run

# Force sync (use with caution)
argocd app sync recommendation-engine-prod --force
```

### Health check failing

```bash
# Check application resources
argocd app resources recommendation-engine-prod

# Check pod status
kubectl get pods -n recommendation-engine

# View logs
kubectl logs -n recommendation-engine -l app.kubernetes.io/name=recommendation-engine
```

## Best Practices

1. **Use version tags in production** - Never use `latest` or branch names
2. **Manual sync for production** - Require human approval for production changes
3. **Test in dev first** - Always deploy to dev environment before production
4. **Monitor health** - Set up alerts for sync failures and health issues
5. **Backup before changes** - Create backups before major version upgrades

## References

- [ArgoCD Documentation](https://argo-cd.readthedocs.io/)
- [GitOps Best Practices](https://www.gitops.tech/)
