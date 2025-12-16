# Notes de version pour XyNginC v1.1.6

## Corrections de bugs

- **Correction critique** : Les modèles de configuration Nginx (`non_ssl_template.conf` et `ssl_template.conf`) contenaient des commentaires au format JavaScript/TypeScript (`/** ... */`), ce qui causait des erreurs lors de l'application des configurations. Ces commentaires ont été remplacés par des commentaires valides pour Nginx (lignes commençant par `#`).
- **Correction CSS** : Les erreurs CSS dans le fichier `error.html` ont été corrigées.

## Améliorations

- Meilleure compatibilité avec les configurations Nginx.
- Meilleure gestion des erreurs lors de l'application des configurations.
- **Injection dynamique de contenu** : Les fichiers HTML (`error.html` et `index.html`) supportent maintenant l'injection dynamique de contenu via des variables (`{{VARIABLE_NAME}}`).
- **Colorisation des logs** : Les logs sont maintenant colorisés pour une meilleure lisibilité (succès en vert, erreurs en rouge, avertissements en jaune, etc.).
- **Suppression des emojis non essentiels** : Les emojis non essentiels ont été supprimés des logs pour une apparence plus professionnelle.

## Instructions de mise à jour

1. Remplacez le binaire `xynginc` par la nouvelle version.
2. Si vous avez des configurations existantes, assurez-vous de les mettre à jour avec les nouveaux modèles corrigés.

## Problèmes connus

Aucun problème connu dans cette version.

## Remerciements

Merci à tous les utilisateurs pour leurs retours et signalements de bugs.
