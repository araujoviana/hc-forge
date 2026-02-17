export const AUTO_VM_UPDATE_COMMAND = `
hc_forge_progress 2 "Startup package update started."
if command -v apt-get >/dev/null 2>&1; then
  hc_forge_progress 8 "Package manager: apt-get"
  export DEBIAN_FRONTEND=noninteractive
  apt-get update
  hc_forge_progress 26 "apt metadata refreshed."
  apt-get -y -o Dpkg::Options::=--force-confnew dist-upgrade
  hc_forge_progress 78 "apt dist-upgrade complete."
  apt-get -y autoremove --purge
  hc_forge_progress 92 "apt autoremove complete."
elif command -v dnf >/dev/null 2>&1; then
  hc_forge_progress 8 "Package manager: dnf"
  dnf -y upgrade --refresh
  hc_forge_progress 92 "dnf upgrade complete."
elif command -v yum >/dev/null 2>&1; then
  hc_forge_progress 8 "Package manager: yum"
  yum -y update
  hc_forge_progress 92 "yum update complete."
elif command -v zypper >/dev/null 2>&1; then
  hc_forge_progress 8 "Package manager: zypper"
  zypper --non-interactive refresh
  hc_forge_progress 26 "zypper refresh complete."
  zypper --non-interactive update
  hc_forge_progress 92 "zypper update complete."
elif command -v pacman >/dev/null 2>&1; then
  hc_forge_progress 8 "Package manager: pacman"
  pacman -Syu --noconfirm
  hc_forge_progress 92 "pacman upgrade complete."
elif command -v apk >/dev/null 2>&1; then
  hc_forge_progress 8 "Package manager: apk"
  apk update
  hc_forge_progress 28 "apk metadata refreshed."
  apk upgrade
  hc_forge_progress 92 "apk upgrade complete."
else
  echo "No supported package manager found for automatic updates."
  exit 2
fi
hc_forge_progress 100 "Startup package update finished."
`.trim();
export const SETUP_GUI_RDP_COMMAND = `
hc_forge_progress 5 "Desktop+RDP setup started."
RDP_USER="\${HC_FORGE_RDP_USER:-hcforge000000}"
RDP_PASSWORD="\${HC_FORGE_RDP_PASSWORD:-}"
if [ -z "\${RDP_PASSWORD}" ]; then
  echo "Missing HC_FORGE_RDP_PASSWORD for Desktop+RDP setup."
  exit 6
fi

hc_dump_xrdp_logs() {
  if [ -f /var/log/xrdp.log ]; then
    tail -n 40 /var/log/xrdp.log | sed 's/^/[hc-forge] xrdp.log: /'
  fi
  if [ -f /var/log/xrdp-sesman.log ]; then
    tail -n 40 /var/log/xrdp-sesman.log | sed 's/^/[hc-forge] xrdp-sesman.log: /'
  fi
  if command -v journalctl >/dev/null 2>&1; then
    journalctl -u xrdp -u xrdp-sesman -n 20 --no-pager 2>/dev/null | sed 's/^/[hc-forge] journal: /'
  fi
}

if command -v apt-get >/dev/null 2>&1; then
  hc_forge_progress 12 "Package manager: apt-get"
  export DEBIAN_FRONTEND=noninteractive
  apt-get update
  hc_forge_progress 26 "apt metadata refreshed."
  apt-get install -y --no-install-recommends xorg xrdp xorgxrdp xterm xfce4 dbus-x11 xauth x11-xserver-utils
  hc_forge_progress 78 "apt packages installed."
elif command -v dnf >/dev/null 2>&1; then
  hc_forge_progress 12 "Package manager: dnf"
  dnf -y install xrdp xorgxrdp xorg-x11-server-Xorg xterm xfce4-session xfce4-panel xfdesktop xfwm4 thunar xorg-x11-xauth || \
    dnf -y groupinstall "Xfce Desktop" || \
    dnf -y install xrdp xterm xorg-x11-xauth
  hc_forge_progress 78 "dnf packages installed."
elif command -v yum >/dev/null 2>&1; then
  hc_forge_progress 12 "Package manager: yum"
  yum -y install xrdp xorgxrdp xterm xfce4-session xfce4-panel xfdesktop xfwm4 thunar xorg-x11-xauth || \
    yum -y groupinstall "Xfce" || \
    yum -y install xrdp xterm xorg-x11-xauth
  hc_forge_progress 78 "yum packages installed."
elif command -v zypper >/dev/null 2>&1; then
  hc_forge_progress 12 "Package manager: zypper"
  zypper --non-interactive refresh
  zypper --non-interactive install -y xrdp xorg-x11-server xterm xfce4-session xfce4-panel xfwm4 thunar xauth || \
    zypper --non-interactive install -y xrdp xorg-x11-server xterm xauth
  hc_forge_progress 78 "zypper packages installed."
elif command -v pacman >/dev/null 2>&1; then
  hc_forge_progress 12 "Package manager: pacman"
  pacman -Syu --noconfirm xorg-server xorg-xinit xterm xfce4 xrdp xorg-xauth
  hc_forge_progress 78 "pacman packages installed."
elif command -v apk >/dev/null 2>&1; then
  hc_forge_progress 12 "Package manager: apk"
  apk update
  apk add xrdp xorg-server xinit xterm xfce4 dbus xauth
  hc_forge_progress 78 "apk packages installed."
else
  echo "No supported package manager found for Desktop+RDP setup."
  exit 2
fi

hc_forge_progress 82 "Preparing RDP login user \${RDP_USER}."
if id -u "\${RDP_USER}" >/dev/null 2>&1; then
  echo "[hc-forge] RDP user \${RDP_USER} already exists."
else
  if command -v useradd >/dev/null 2>&1; then
    useradd -m -s /bin/bash "\${RDP_USER}" || useradd -m "\${RDP_USER}"
  elif command -v adduser >/dev/null 2>&1; then
    adduser --disabled-password --gecos "" "\${RDP_USER}" >/dev/null 2>&1 || adduser -D "\${RDP_USER}"
  else
    echo "Cannot create RDP user: no useradd/adduser command found."
    exit 7
  fi
fi
if command -v usermod >/dev/null 2>&1; then
  usermod -s /bin/bash "\${RDP_USER}" || true
fi
echo "\${RDP_USER}:\${RDP_PASSWORD}" | chpasswd
hc_forge_progress 86 "RDP user credentials set."

RDP_HOME="\$(getent passwd "\${RDP_USER}" | cut -d: -f6 || true)"
if [ -z "\${RDP_HOME}" ]; then
  RDP_HOME="/home/\${RDP_USER}"
fi
mkdir -p "\${RDP_HOME}"
cat > "\${RDP_HOME}/.xsession" <<'EOF_HC_FORGE_RDP'
#!/bin/sh
export XDG_SESSION_TYPE=x11
export XDG_CURRENT_DESKTOP=XFCE
export XDG_SESSION_DESKTOP=xfce
export DESKTOP_SESSION=xfce
unset DBUS_SESSION_BUS_ADDRESS
unset XDG_RUNTIME_DIR
if command -v dbus-launch >/dev/null 2>&1 && command -v xfce4-session >/dev/null 2>&1; then
  exec dbus-launch --exit-with-session xfce4-session
elif command -v startxfce4 >/dev/null 2>&1; then
  exec startxfce4
elif command -v xfce4-session >/dev/null 2>&1; then
  exec xfce4-session
else
  exec xterm
fi
EOF_HC_FORGE_RDP
cp "\${RDP_HOME}/.xsession" "\${RDP_HOME}/.Xsession"
chown "\${RDP_USER}:\${RDP_USER}" "\${RDP_HOME}/.xsession" "\${RDP_HOME}/.Xsession" || true
touch "\${RDP_HOME}/.Xauthority"
chown "\${RDP_USER}:\${RDP_USER}" "\${RDP_HOME}/.Xauthority" || true
chmod 700 "\${RDP_HOME}" || true
chmod 755 "\${RDP_HOME}/.xsession" "\${RDP_HOME}/.Xsession" || true
chmod 600 "\${RDP_HOME}/.Xauthority" || true
hc_forge_progress 88 "XFCE session profile configured for \${RDP_USER}."

if [ -d /etc/X11 ]; then
  cat > /etc/X11/Xwrapper.config <<'EOF_HC_FORGE_XWRAP'
allowed_users=anybody
needs_root_rights=yes
EOF_HC_FORGE_XWRAP
fi
hc_forge_progress 89 "Xorg wrapper permissions configured."

if [ -f /etc/xrdp/startwm.sh ]; then
  cat > /etc/xrdp/startwm.sh <<'EOF_HC_FORGE_STARTWM'
#!/bin/sh
if [ -r /etc/profile ]; then
  . /etc/profile
fi
if [ -r "$HOME/.profile" ]; then
  . "$HOME/.profile"
fi
unset DBUS_SESSION_BUS_ADDRESS
unset XDG_RUNTIME_DIR
if [ -r "$HOME/.xsession" ]; then
  exec /bin/sh "$HOME/.xsession"
fi
if command -v startxfce4 >/dev/null 2>&1; then
  exec startxfce4
fi
if command -v xfce4-session >/dev/null 2>&1; then
  exec xfce4-session
fi
if [ -x /etc/X11/Xsession ]; then
  exec /etc/X11/Xsession
fi
exec xterm
EOF_HC_FORGE_STARTWM
  chmod 755 /etc/xrdp/startwm.sh || true
fi
hc_forge_progress 91 "XRDP session launcher configured."

if [ -f /etc/xrdp/xrdp.ini ]; then
  cp /etc/xrdp/xrdp.ini /etc/xrdp/xrdp.ini.hc-forge.bak 2>/dev/null || true
  awk '
  BEGIN {
    section = ""
    globals_port_set = 0
    xorg_port_set = 0
    xorg_ip_set = 0
  }
  function flush_section() {
    if (section == "globals" && !globals_port_set) {
      print "port=3389"
      globals_port_set = 1
      return
    }
    if (section == "xorg") {
      if (!xorg_port_set) {
        print "port=-1"
        xorg_port_set = 1
      }
      if (!xorg_ip_set) {
        print "ip=127.0.0.1"
        xorg_ip_set = 1
      }
    }
  }
  {
    line = $0
    trimmed = $0
    sub(/^[[:space:]]+/, "", trimmed)

    if (trimmed ~ /^\[/) {
      flush_section()
      header = tolower(trimmed)
      gsub(/[[:space:]]/, "", header)
      if (header == "[globals]") {
        section = "globals"
        globals_port_set = 0
      } else if (header == "[xorg]") {
        section = "xorg"
        xorg_port_set = 0
        xorg_ip_set = 0
      } else {
        section = ""
      }
      print line
      next
    }

    if (section == "globals" && trimmed ~ /^port[[:space:]]*=/) {
      print "port=3389"
      globals_port_set = 1
      next
    }

    if (section == "xorg" && trimmed ~ /^port[[:space:]]*=/) {
      print "port=-1"
      xorg_port_set = 1
      next
    }

    if (section == "xorg" && trimmed ~ /^ip[[:space:]]*=/) {
      print "ip=127.0.0.1"
      xorg_ip_set = 1
      next
    }

    print line
  }
  END {
    flush_section()
  }
  ' /etc/xrdp/xrdp.ini > /tmp/hc_forge_xrdp.ini
  mv /tmp/hc_forge_xrdp.ini /etc/xrdp/xrdp.ini
  if command -v grep >/dev/null 2>&1; then
    grep -nE '^\[|^[[:space:]]*port=|^[[:space:]]*ip=' /etc/xrdp/xrdp.ini | sed 's/^/[hc-forge] xrdp.ini: /'
  fi
fi
hc_forge_progress 92 "XRDP configured on port 3389."
echo "[hc-forge] RDP user: \${RDP_USER} (password matches VM admin password)."

if id -u xrdp >/dev/null 2>&1 && getent group ssl-cert >/dev/null 2>&1; then
  usermod -aG ssl-cert xrdp || true
fi

if command -v systemctl >/dev/null 2>&1; then
  systemctl enable xrdp || true
  systemctl enable xrdp-sesman || true
  systemctl restart xrdp || true
  systemctl restart xrdp-sesman || true
elif command -v rc-update >/dev/null 2>&1; then
  rc-update add xrdp default || true
  rc-service xrdp restart || true
elif command -v service >/dev/null 2>&1; then
  service xrdp restart || true
  service xrdp-sesman restart || true
fi
hc_forge_progress 94 "XRDP services restarted."

if command -v ss >/dev/null 2>&1; then
  if ! ss -ltn | grep -q ':3389'; then
    echo "[hc-forge] XRDP is not listening on TCP 3389."
    hc_dump_xrdp_logs
    exit 8
  fi
  if ! ss -ltn | grep -q ':3350'; then
    echo "[hc-forge] XRDP sesman is not listening on TCP 3350."
    hc_dump_xrdp_logs
    exit 9
  fi
  hc_forge_progress 96 "XRDP and sesman are listening (3389/3350)."
fi

if ! command -v startxfce4 >/dev/null 2>&1 && ! command -v xfce4-session >/dev/null 2>&1; then
  echo "[hc-forge] XFCE command not found after installation."
  hc_dump_xrdp_logs
  exit 10
fi

hc_forge_progress 100 "Desktop+RDP setup finished."
`.trim();
