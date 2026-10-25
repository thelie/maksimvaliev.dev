#!/bin/sh
# certbot deploy hook: /etc/letsencrypt/renewal-hooks/deploy/pingap-maksimvaliev.sh
# After issuing or renewing maksimvaliev.dev, copies the certificate to /etc/pingap/tls/maksimvaliev.dev,
# where pingap reads it (the Let's Encrypt directory is root-only). pingap picks up new files by itself.
# The other site on the server has its own hook (pingap.sh); each one skips the other's domain.
set -eu
src=/etc/letsencrypt/live/maksimvaliev.dev
dst=/etc/pingap/tls/maksimvaliev.dev
case " ${RENEWED_DOMAINS:-maksimvaliev.dev} " in *" maksimvaliev.dev "*) ;; *) exit 0 ;; esac

install -d -m 0750 -o root -g pingap "$dst"
# Through temporary files and mv, so pingap never reads a half-written file
install -m 0640 -o root -g pingap "$src/privkey.pem" "$dst/.privkey.pem.new"
install -m 0644 -o root -g pingap "$src/fullchain.pem" "$dst/.fullchain.pem.new"
mv -f "$dst/.privkey.pem.new" "$dst/privkey.pem"
mv -f "$dst/.fullchain.pem.new" "$dst/fullchain.pem"
