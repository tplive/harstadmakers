#!/usr/bin/env bash
set -euo pipefail

# --- CONFIGURATION ---
REPO_OWNER="tplive"
REPO_NAME="harstadmakers"
TTL_VERSION="${TTL_VERSION:-latest}"
POCKETBASE_VERSION="0.41.1"  # Or detect latest
# --- END CONFIG ---

# Detect platform
ARCH=$(uname -m)
case "$ARCH" in
    aarch64) TTL_API_ARCH="aarch64-unknown-linux-gnu" ;;
    x86_64)  TTL_API_ARCH="x86_64-unknown-linux-gnu" ;;
    *)
        echo "Unsupported architecture: $ARCH"
        exit 1
        ;;
esac

# Root and service setup (idempotent)
setup_system() {
    # Create service users if they don't exist
    for user in svc-pocketbase svc-ttl-api svc-ttl-frontend; do
        if ! id "$user" &>/dev/null; then
            sudo adduser --system --group --shell /usr/sbin/nologin --home "/srv/${user#svc-}" "$user"
        fi
    done

    # Create groups
    sudo groupadd --system --force ttl-certs 2>/dev/null || true

    # Create directories
    for dir in /srv/pocketbase /srv/ttl-api /srv/ttl-frontend /srv/ttl-certs /srv/deno; do
        sudo install -d -o root -g root -m 0755 "$dir"
    done

    # Cert directory
    sudo chown root:ttl-certs /srv/ttl-certs
    sudo chmod 750 /srv/ttl-certs
}

# Install PocketBase (idempotent)
install_pocketbase() {
    local pb_bin="/srv/pocketbase/pocketbase"
    if [[ -x "$pb_bin" && "$($pb_bin --version 2>/dev/null || echo '')" == "$POCKETBASE_VERSION" ]]; then
        echo "PocketBase $POCKETBASE_VERSION already installed"
        return
    fi

    echo "Installing PocketBase $POCKETBASE_VERSION..."
    # Map architecture for PocketBase (uses arm64/amd64, not aarch64/x86_64)
    local pb_arch
    case "$ARCH" in
        aarch64) pb_arch="arm64" ;;
        x86_64)  pb_arch="amd64" ;;
        *) 
            echo "Unsupported architecture for PocketBase: $ARCH"
            exit 1
            ;;
    esac

    local 
    local url="https://ganigeorgiev.com/pocketbase/pocketbase/releases/download/v$POCKETBASE_VERSION/pocketbase_${POCKETBASE_VERSION}_linux_$(pb_arch).zip"
    local tmp=$(mktemp -d)
    curl -fsSL "$url" -o "$tmp/pocketbase.zip"
    unzip -d "$tmp" "$tmp/pocketbase.zip"
    sudo install -o svc-pocketbase -g svc-pocketbase -m 0755 "$tmp/pocketbase" "$pb_bin"
    rm -rf "$tmp"
    echo "PocketBase installed"
}

# Install Deno (idempotent)
install_deno() {
    local deno_bin="/srv/deno/deno"
    if [[ -x "$deno_bin" ]]; then
        echo "Deno already installed"
        return
    fi

    echo "Installing Deno..."
    local tmp=$(mktemp -d)
    curl -fsSL https://deno.land/install.sh | DENO_INSTALL="$tmp" sh -s -- --yes
    sudo install -o root -g root -m 0755 "$tmp/bin/deno" "$deno_bin"
    sudo install -d -o svc-ttl-frontend -g svc-ttl-frontend -m 0750 /srv/deno/.cache
    rm -rf "$tmp"
    echo "Deno installed"
}

# Install ttl-api binary (idempotent)
install_ttl_api() {
    local api_bin="/srv/ttl-api/ttl-api"
    if [[ "$TTL_VERSION" == "latest" ]]; then
        # Get latest release tag from GitHub
        TTL_VERSION=$(curl -s "https://api.github.com/repos/$REPO_OWNER/$REPO_NAME/releases/latest" | jq -r '.tag_name')
    fi

    if [[ -x "$api_bin" ]]; then
        echo "ttl-api already installed at $api_bin"
        return
    fi

    echo "Installing ttl-api v$TTL_VERSION..."
    local url="https://github.com/$REPO_OWNER/$REPO_NAME/releases/download/$TTL_VERSION/ttl-api-$TTL_API_ARCH"
    sudo curl -fsSL -o "$api_bin" "$url"
    sudo chown svc-ttl-api:svc-ttl-api "$api_bin"
    sudo chmod 0755 "$api_bin"
    echo "ttl-api installed"
}

# Install frontend files (idempotent)
install_frontend() {
    if [[ -d "/srv/ttl-frontend/index.html" ]]; then
        echo "Frontend already installed"
        return
    fi

    echo "Installing frontend..."
    sudo curl -fsSL https://github.com/$REPO_OWNER/$REPO_NAME/tarball/$TTL_VERSION \
        | sudo tar -xz -C /tmp --strip-components=1 "*/ttl/public"
    sudo cp -r /tmp/ttl/public/* /srv/ttl-frontend/
    sudo chown -R svc-ttl-frontend:svc-ttl-frontend /srv/ttl-frontend
    rm -rf /tmp/ttl
    echo "Frontend installed"
}

# Install systemd services (idempotent)
install_services() {
    for service in pocketbase ttl-api ttl-frontend; do
        local src="ini_service_files/${service}.service"
        local dst="/etc/systemd/system/${service}.service"
        if [[ -f "$src" ]]; then
            sudo install -o root -g root -m 0644 "$src" "$dst"
            sudo systemctl daemon-reload
            sudo systemctl enable --now "$service"
        fi
    done
}

# Main
main() {
    echo "=== TTL Production Setup ==="

    # Ensure running as root or with sudo
    if [[ $EUID -ne 0 ]]; then
        echo "Please run with sudo"
        exit 1
    fi

    setup_system
    install_pocketbase
    install_deno
    install_ttl_api
    install_frontend
    install_services

    echo ""
    echo "✅ Installation complete!"
    echo ""
    echo "Next steps:"
    echo "1. Import PocketBase schema: pb_schema.json"
    echo "2. Configure certificates in /srv/ttl-certs/"
    echo "3. Configure /srv/ttl-api/configuration.json"
    echo "4. Add svc-ttl-api to gpio group: usermod -aG gpio svc-ttl-api"
    echo "5. Restart services: systemctl restart pocketbase ttl-api ttl-frontend"
}

main "$@"