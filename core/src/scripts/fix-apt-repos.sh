#!/bin/bash

# Script pour nettoyer les dépôts APT problématiques sur Kali Linux
# Ce script résout les erreurs de dépôts et permet d'installer nginx

echo "> Nettoyage des dépôts APT problématiques..."
echo ""

# Couleurs pour les messages
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Vérifier les permissions sudo
if [ "$EUID" -ne 0 ]; then 
    echo -e "${RED}❌ Ce script doit être exécuté avec sudo${NC}"
    echo "Usage: sudo bash fix-apt-repos.sh"
    exit 1
fi

echo -e "${YELLOW}📋 Étape 1: Sauvegarde des sources actuelles${NC}"
cp -r /etc/apt/sources.list.d /etc/apt/sources.list.d.backup.$(date +%Y%m%d_%H%M%S)
echo -e "${GREEN}✓ Sauvegarde créée${NC}"
echo ""

echo -e "${YELLOW}📋 Étape 2: Désactivation des dépôts problématiques${NC}"

# Désactiver le dépôt PostgreSQL/pgAdmin problématique
if [ -f /etc/apt/sources.list.d/pgdg.list ]; then
    echo "  → Désactivation de pgdg.list"
    mv /etc/apt/sources.list.d/pgdg.list /etc/apt/sources.list.d/pgdg.list.disabled
fi

# Désactiver le dépôt Docker problématique pour Kali
if [ -f /etc/apt/sources.list.d/docker.list ]; then
    echo "  → Désactivation de docker.list"
    mv /etc/apt/sources.list.d/docker.list /etc/apt/sources.list.d/docker.list.disabled
fi

# Désactiver le dépôt LLVM avec signature expirante
if [ -f /etc/apt/sources.list.d/llvm.list ]; then
    echo "  → Désactivation de llvm.list"
    mv /etc/apt/sources.list.d/llvm.list /etc/apt/sources.list.d/llvm.list.disabled
fi

# Chercher d'autres fichiers qui pourraient contenir ces URLs
for file in /etc/apt/sources.list.d/*.list; do
    if [ -f "$file" ]; then
        if grep -q "ftp.postgresql.org" "$file" 2>/dev/null || \
           grep -q "download.docker.com/linux/ubuntu" "$file" 2>/dev/null || \
           grep -q "apt.llvm.org" "$file" 2>/dev/null; then
            echo "  → Désactivation de $(basename $file)"
            mv "$file" "$file.disabled"
        fi
    fi
done

echo -e "${GREEN}✓ Dépôts problématiques désactivés${NC}"
echo ""

echo -e "${YELLOW}📋 Étape 3: Vérification des dépôts Kali principaux${NC}"

# S'assurer que les dépôts Kali officiels sont présents
if ! grep -q "deb http://http.kali.org/kali kali-rolling main" /etc/apt/sources.list; then
    echo "  → Ajout des dépôts Kali officiels"
    cat >> /etc/apt/sources.list <<EOF

# Dépôts Kali officiels
deb http://http.kali.org/kali kali-rolling main contrib non-free non-free-firmware
deb-src http://http.kali.org/kali kali-rolling main contrib non-free non-free-firmware
EOF
fi

echo -e "${GREEN}✓ Dépôts Kali vérifiés${NC}"
echo ""

echo -e "${YELLOW}📋 Étape 4: Mise à jour de la liste des paquets${NC}"
apt-get update 2>&1 | grep -v "^W:" | grep -v "^N:" || true
echo -e "${GREEN}✓ Liste des paquets mise à jour${NC}"
echo ""

echo -e "${YELLOW}📋 Étape 5: Installation de nginx${NC}"
apt-get install -y nginx
if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓ Nginx installé avec succès${NC}"
else
    echo -e "${RED}❌ Échec de l'installation de nginx${NC}"
    exit 1
fi
echo ""

echo -e "${YELLOW}📋 Étape 6: Activation et démarrage de nginx${NC}"
systemctl enable nginx
systemctl start nginx
echo -e "${GREEN}✓ Nginx activé et démarré${NC}"
echo ""

echo -e "${GREEN}🎉 Nettoyage terminé avec succès !${NC}"
echo ""
echo "📝 Résumé:"
echo "  • Dépôts problématiques sauvegardés dans /etc/apt/sources.list.d.backup.*"
echo "  • Dépôts problématiques désactivés (fichiers .disabled)"
echo "  • Nginx installé et configuré"
echo ""
echo "💡 Pour réactiver un dépôt désactivé:"
echo "   sudo mv /etc/apt/sources.list.d/nom_du_depot.list.disabled /etc/apt/sources.list.d/nom_du_depot.list"
echo ""
echo "✅ Vous pouvez maintenant relancer: npm run dev"