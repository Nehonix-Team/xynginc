# Structure Modulaire de XyNginC

## Vue d'ensemble

Le fichier `main.rs` a été modularisé en 12 modules distincts, organisés dans le dossier `src/mods/`. Cette refactorisation améliore la maintenabilité, la lisibilité et la réutilisabilité du code.

## Structure des Modules

```
core/src/
├── main.rs                 # Point d'entrée principal (60 lignes)
├── requirements.rs         # Module existant (inchangé)
└── mods/
    ├── mod.rs             # Déclaration des modules
    ├── logger.rs          # Fonctions de logging coloré
    ├── constants.rs       # Constantes et templates embarqués
    ├── models.rs          # Structures de données (Config, DomainConfig)
    ├── cli.rs             # Interface CLI (clap)
    ├── backup.rs          # Gestion des backups
    ├── nginx.rs           # Opérations nginx (test, reload, status)
    ├── ssl.rs             # Gestion SSL/certbot
    ├── cleanup.rs         # Nettoyage des configurations cassées
    ├── config.rs          # Génération et gestion des configurations
    ├── domain.rs          # Gestion des domaines (add, remove, list)
    ├── apply.rs           # Application de configuration JSON
    └── check.rs           # Vérification des prérequis système
```

## Description des Modules

### 1. **logger.rs** (424 bytes)

Fonctions utilitaires pour l'affichage coloré dans le terminal :

- `log_info()` - Messages informatifs (blanc)
- `log_success()` - Messages de succès (vert)
- `log_warning()` - Avertissements (jaune)
- `log_error()` - Erreurs (rouge)
- `log_step()` - Étapes importantes (bleu gras)

### 2. **constants.rs** (544 bytes)

Constantes globales et templates embarqués :

- Templates de configuration : `NON_SSL_TEMPLATE`, `SSL_TEMPLATE`
- Templates HTML : `ERROR_HTML`, `INDEX_HTML`
- Chemins système : `NGINX_SITES_AVAILABLE`, `NGINX_SITES_ENABLED`, `BACKUP_DIR`

### 3. **models.rs** (500 bytes)

Structures de données sérialisables :

- `Config` - Configuration globale avec liste de domaines
- `DomainConfig` - Configuration d'un domaine individuel

### 4. **cli.rs** (1.8 KB)

Définition de l'interface en ligne de commande avec clap :

- Structure `Cli` principale
- Enum `Commands` avec toutes les sous-commandes

### 5. **backup.rs** (3.8 KB)

Gestion complète des backups :

- `create_backup()` - Créer un backup horodaté
- `restore_backup()` - Restaurer un backup spécifique
- `restore_latest_backup()` - Restaurer le backup le plus récent
- `list_backups()` - Lister tous les backups disponibles
- `copy_directory()` - Copier récursivement des répertoires

### 6. **nginx.rs** (1.9 KB)

Opérations sur le service nginx :

- `test_nginx()` - Tester la configuration nginx
- `reload_nginx()` - Recharger nginx
- `show_status()` - Afficher le statut complet du système

### 7. **ssl.rs** (896 bytes)

Gestion des certificats SSL :

- `setup_ssl()` - Obtenir un certificat Let's Encrypt via certbot

### 8. **cleanup.rs** (3.4 KB)

Détection et nettoyage des configurations problématiques :

- `detect_broken_configs()` - Analyser les erreurs nginx
- `clean_broken_configs()` - Nettoyer les configurations cassées
- `remove_config_files()` - Supprimer les fichiers de configuration

### 9. **config.rs** (5.9 KB)

Génération et gestion des configurations :

- `generate_nginx_config()` - Générer une configuration nginx
- `load_template()` - Charger un template embarqué
- `replace_template_variables()` - Remplacer les variables dans les templates
- `ensure_error_page_exists()` - Créer la page d'erreur personnalisée
- `ensure_index_page_exists()` - Créer la page d'index XyNginC
- `config_exists()` - Vérifier si une configuration existe

### 10. **domain.rs** (3.1 KB)

Gestion des domaines :

- `list_domains()` - Lister tous les domaines configurés
- `add_domain()` - Ajouter un nouveau domaine
- `remove_domain()` - Supprimer un domaine
- `enable_site()` - Activer un site (créer le symlink)

### 11. **apply.rs** (3.6 KB)

Application de configuration complète depuis JSON :

- `apply_config()` - Processus complet d'application de configuration
  - Lecture du fichier JSON ou stdin
  - Création de backup
  - Détection et nettoyage des configs cassées
  - Application des nouvelles configurations
  - Test et validation
  - Rollback automatique en cas d'erreur
  - Reload optionnel

### 12. **check.rs** (2.0 KB)

Vérification des prérequis système :

- `check_requirements()` - Vérifier nginx, certbot et les répertoires nécessaires

## Avantages de cette Modularisation

### ✅ Maintenabilité

- Chaque module a une responsabilité unique et claire
- Facilite la localisation et la correction de bugs
- Simplifie l'ajout de nouvelles fonctionnalités

### ✅ Lisibilité

- Le fichier `main.rs` est passé de ~895 lignes à ~60 lignes
- Code organisé logiquement par domaine fonctionnel
- Imports explicites montrant les dépendances

### ✅ Réutilisabilité

- Les fonctions peuvent être facilement réutilisées entre modules
- Séparation claire entre logique métier et présentation

### ✅ Testabilité

- Chaque module peut être testé indépendamment
- Facilite l'écriture de tests unitaires

### ✅ Évolutivité

- Facile d'ajouter de nouveaux modules
- Structure claire pour les futures fonctionnalités

## Migration Effectuée

### Avant

```
main.rs - 895 lignes (27.4 KB)
├── Toutes les fonctions mélangées
├── Constantes, structures, CLI, logique métier
└── Difficile à naviguer et maintenir
```

### Après

```
main.rs - 60 lignes (1.5 KB)
└── mods/ - 12 modules spécialisés (28.5 KB total)
    ├── Séparation claire des responsabilités
    ├── Navigation intuitive
    └── Facile à maintenir et étendre
```

## Compatibilité

✅ **Aucune fonctionnalité n'a été modifiée ou supprimée**

- Toutes les commandes CLI fonctionnent exactement comme avant
- Les templates embarqués sont toujours inclus dans le binaire
- Le comportement de l'application est identique
- La compilation réussit sans warnings ni erreurs

## Compilation

```bash
cd core
cargo build --release
```

Le binaire compilé est disponible dans `target/release/xynginc` et fonctionne exactement comme avant la modularisation.
