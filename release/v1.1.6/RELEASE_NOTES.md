# Notes de version pour XyNginC v1.1.6

## Corrections de bugs

- **Correction critique** : Les modèles de configuration Nginx (`non_ssl_template.conf` et `ssl_template.conf`) contenaient des commentaires au format JavaScript/TypeScript (`/** ... */`), ce qui causait des erreurs lors de l'application des configurations. Ces commentaires ont été remplacés par des commentaires valides pour Nginx (lignes commençant par `#`).

## Améliorations

- Meilleure compatibilité avec les configurations Nginx.
- Meilleure gestion des erreurs lors de l'application des configurations.

## Instructions de mise à jour

1. Remplacez le binaire `xynginc` par la nouvelle version.
2. Si vous avez des configurations existantes, assurez-vous de les mettre à jour avec les nouveaux modèles corrigés.

## Problèmes connus

Aucun problème connu dans cette version.

## Remerciements

Merci à tous les utilisateurs pour leurs retours et signalements de bugs.
