#!/bin/bash

# pCloud Sync for Thetawave Assets
# Downloads thetawave/data and thetawave/media from pCloud to assets/
# Uploads changed files from assets/ to pCloud

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PCLOUD_DIR="$SCRIPT_DIR"
VENV_DIR="$SCRIPT_DIR/venv"

# Setup virtual environment
setup_venv() {
    if [[ ! -d "$VENV_DIR" ]]; then
        echo "Creating virtual environment..."
        python3 -m venv "$VENV_DIR"
        echo "Installing dependencies..."
        "$VENV_DIR/bin/pip" install -r "$SCRIPT_DIR/requirements.txt"
    fi
}

# Activate virtual environment
activate_venv() {
    source "$VENV_DIR/bin/activate"
}

# Check if credentials are set
check_env() {
    if [[ -z "$PCLOUD_USERNAME" || -z "$PCLOUD_PASSWORD" ]]; then
        echo "Error: Set PCLOUD_USERNAME and PCLOUD_PASSWORD environment variables"
        echo "Or create a .env file with your credentials in $SCRIPT_DIR/.env"
        echo "Example .env file:"
        echo "PCLOUD_USERNAME=your.email@example.com"
        echo "PCLOUD_PASSWORD=\"your_password_here\""
        exit 1
    fi
}

# Source .env file if it exists
if [[ -f "$SCRIPT_DIR/.env" ]]; then
    set -a  # Automatically export all variables
    source "$SCRIPT_DIR/.env"
    set +a  # Stop auto-exporting
fi

case "${1:-}" in
    "download")
        setup_venv
        activate_venv
        check_env
        echo "Downloading assets from pCloud..."
        python "$SCRIPT_DIR/pcloud_sync.py" download
        ;;
    "upload")
        setup_venv
        activate_venv
        check_env
        if [[ "${2:-}" == "--execute" ]]; then
            echo "Uploading assets to pCloud..."
            python "$SCRIPT_DIR/pcloud_sync.py" upload --execute
        else
            echo "Previewing upload to pCloud (dry run)..."
            python "$SCRIPT_DIR/pcloud_sync.py" upload
        fi
        ;;

    "test")
        setup_venv
        activate_venv
        check_env
        echo "Testing pCloud connection..."
        python "$SCRIPT_DIR/pcloud_sync.py" test
        ;;
    *)
        echo "Usage: $0 {download|upload|test}"
        echo ""
        echo "Commands:"
        echo "  download             Download thetawave/data and thetawave/media to assets/"
        echo "  upload               Preview what would be uploaded (dry run)"
        echo "  upload --execute     Actually upload assets/data and assets/media to pCloud"
        echo "  test                 Test pCloud connection and list directory contents"
        echo ""
        echo "Set PCLOUD_USERNAME and PCLOUD_PASSWORD environment variables"
        exit 1
        ;;
esac
