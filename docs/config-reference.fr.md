# Référence de configuration (FR)

Schéma de configuration canonique:

- [`../src/config/schema.rs`](../src/config/schema.rs)

Chargement/fusion de config:

- [`../src/config/mod.rs`](../src/config/mod.rs)

Clés plugin récentes à connaître :

- `[plugins].marketplace_enabled` (désactivé par défaut, requis pour les sources HTTP(S))
- `[plugins].allowed_permissions` (liste d’autorisations acceptées à l’installation)
