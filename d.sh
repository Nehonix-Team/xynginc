sudo systemctl stop nginx
sudo systemctl disable nginx
sudo apt purge nginx nginx-common nginx-core nginx-full -y
sudo apt autoremove --purge -y
sudo apt autoclean
sudo apt purge certbot python3-certbot-nginx -y
sudo apt autoremove --purge -y
