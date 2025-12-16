# XyNginC v1.1.6

XyNginC est un outil en ligne de commande pour simplifier la gestion des configurations Nginx et SSL.

## Changements dans cette version

- **Correction de bug** : Les commentaires dans les modèles de configuration (`non_ssl_template.conf` et `ssl_template.conf`) ont été corrigés pour utiliser le format valide pour Nginx (`#` au lieu de `/** ... */`).
- **Amélioration** : Meilleure compatibilité avec les configurations Nginx.

## Installation

1. Téléchargez le binaire depuis les releases ou compilez depuis la source.
2. Placez le binaire dans `/usr/local/bin/` ou un autre répertoire dans votre PATH.
3. Assurez-vous que Nginx et Certbot sont installés.

## Utilisation

```bash
# Appliquer une configuration
echo '{"domains":[{"domain":"example.com","port":3000,"ssl":false,"email":"admin@example.com","host":"localhost"}],"auto_reload":true}' | sudo xynginc apply --config -

# Lister les domaines configurés
sudo xynginc list

# Tester la configuration Nginx
sudo xynginc test
```

## Configuration

Les modèles de configuration sont inclus dans le binaire et peuvent être personnalisés en modifiant les fichiers dans `core/src/configs/`.

## Support

Pour toute question ou problème, veuillez ouvrir une issue sur le dépôt GitHub.