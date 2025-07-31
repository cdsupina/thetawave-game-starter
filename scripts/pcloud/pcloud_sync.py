#!/usr/bin/env python3
"""
pCloud Sync for Thetawave Assets using WebDAV

This script handles syncing specific directories between pCloud and local assets:
- Downloads thetawave/data and thetawave/media from pCloud to assets/
- Uploads changed/added files from assets/ to pCloud

Usage:
    python3 pcloud_sync.py download
    python3 pcloud_sync.py upload [--execute]
    python3 pcloud_sync.py test

Environment variables:
    PCLOUD_USERNAME - Your pCloud email
    PCLOUD_PASSWORD - Your pCloud password
"""

import os
import sys
import requests
from pathlib import Path
from urllib.parse import quote
import time
import hashlib

def get_webdav_session():
    """Initialize WebDAV session with pCloud"""
    username = os.getenv('PCLOUD_USERNAME')
    password = os.getenv('PCLOUD_PASSWORD')

    if not username or not password:
        print("Error: Set PCLOUD_USERNAME and PCLOUD_PASSWORD environment variables")
        sys.exit(1)

    # pCloud WebDAV endpoint (eapi for EU, api for US)
    base_url = "https://webdav.pcloud.com"

    session = requests.Session()
    session.auth = (username, password)

    # Test connection
    try:
        response = session.request('PROPFIND', base_url, headers={'Depth': '0'})
        if response.status_code not in [200, 207]:
            print(f"Error: Failed to connect to pCloud WebDAV. Status: {response.status_code}")
            sys.exit(1)
    except Exception as e:
        print(f"Error connecting to pCloud: {e}")
        sys.exit(1)

    return session, base_url

def get_webdav_url(base_url, path):
    """Convert path to WebDAV URL"""
    if not path.startswith('/'):
        path = '/' + path
    encoded_path = quote(path, safe='/')
    return f"{base_url}{encoded_path}"

def list_webdav_directory(session, base_url, remote_path, max_retries=3):
    """List files and directories in WebDAV directory with retry logic"""
    url = get_webdav_url(base_url, remote_path)

    for attempt in range(max_retries):
        try:
            response = session.request('PROPFIND', url, headers={'Depth': '1'}, timeout=60)
            if response.status_code == 207:
                content = response.text
                files = []
                directories = []

                # Parse XML response by splitting into individual response blocks
                import re
                from urllib.parse import unquote

                # Split content by <D:response> tags
                response_blocks = content.split('<D:response')[1:]  # Skip first empty element

                for block in response_blocks:
                    # Extract href
                    href_match = re.search(r'<D:href>([^<]+)</D:href>', block)
                    if not href_match:
                        continue

                    href = href_match.group(1)
                    decoded_href = unquote(href)

                    # Skip the directory itself
                    if decoded_href == remote_path or decoded_href == remote_path + '/':
                        continue

                    # Check if this is a collection (directory)
                    is_collection = '<D:collection/>' in block

                    if is_collection:
                        # This is a directory
                        dir_name = decoded_href.rstrip('/').split('/')[-1]
                        if dir_name:
                            directories.append(dir_name)
                            print(f"    Found directory: {dir_name}")
                    else:
                        # This is a file
                        filename = decoded_href.split('/')[-1]
                        if filename and not filename.endswith('/'):
                            files.append(filename)
                            print(f"    Found file: {filename}")

                return files, directories
            else:
                print(f"Error listing directory {remote_path}: {response.status_code}")
                if attempt < max_retries - 1:
                    print(f"    Retrying in 2 seconds... (attempt {attempt + 2}/{max_retries})")
                    time.sleep(2)
                continue
        except Exception as e:
            print(f"Error listing directory {remote_path}: {e}")
            if attempt < max_retries - 1:
                print(f"    Retrying in 2 seconds... (attempt {attempt + 2}/{max_retries})")
                time.sleep(2)
            continue

    print(f"Failed to list directory {remote_path} after {max_retries} attempts")
    return [], []

def download_webdav_file(session, base_url, remote_file, local_file, max_retries=3):
    """Download a single file via WebDAV with retry logic"""
    url = get_webdav_url(base_url, remote_file)

    for attempt in range(max_retries):
        try:
            response = session.get(url, timeout=120)  # Increased timeout for large files
            if response.status_code == 200:
                # Create local directory if needed
                local_file.parent.mkdir(parents=True, exist_ok=True)

                # Get file size from response headers
                file_size = response.headers.get('content-length')
                if file_size:
                    print(f"    ({int(file_size) / 1024 / 1024:.1f} MB)")

                with open(local_file, 'wb') as f:
                    f.write(response.content)
                return True
            else:
                print(f"  Error downloading {remote_file}: {response.status_code}")
                if attempt < max_retries - 1:
                    print(f"    Retrying in 2 seconds... (attempt {attempt + 2}/{max_retries})")
                    time.sleep(2)
                continue
        except Exception as e:
            print(f"  Error downloading {remote_file}: {e}")
            if attempt < max_retries - 1:
                print(f"    Retrying in 2 seconds... (attempt {attempt + 2}/{max_retries})")
                time.sleep(2)
            continue

    print(f"  Failed to download {remote_file} after {max_retries} attempts")
    return False

def get_file_hash(file_path):
    """Get MD5 hash of a file"""
    hash_md5 = hashlib.md5()
    with open(file_path, "rb") as f:
        for chunk in iter(lambda: f.read(4096), b""):
            hash_md5.update(chunk)
    return hash_md5.hexdigest()

def get_remote_file_info(session, base_url, remote_file, max_retries=3):
    """Get remote file info (size, last modified) via WebDAV PROPFIND with retry logic"""
    url = get_webdav_url(base_url, remote_file)

    for attempt in range(max_retries):
        try:
            response = session.request('PROPFIND', url, headers={'Depth': '0'}, timeout=60)
            if response.status_code == 207:
                content = response.text
                # Extract file size and last modified from XML
                import re
                size_match = re.search(r'<lp1:getcontentlength>(\d+)</lp1:getcontentlength>', content)
                modified_match = re.search(r'<lp1:getlastmodified>([^<]+)</lp1:getlastmodified>', content)

                size = int(size_match.group(1)) if size_match else None
                modified = modified_match.group(1) if modified_match else None

                return {'size': size, 'modified': modified}
            elif response.status_code == 404:
                # File doesn't exist
                return None
            else:
                if attempt < max_retries - 1:
                    time.sleep(1)
                continue
        except Exception as e:
            if attempt < max_retries - 1:
                time.sleep(1)
            continue
    return None

def should_upload_file(session, base_url, local_file, remote_file):
    """Check if local file should be uploaded (new or changed)"""
    # Get remote file info
    remote_info = get_remote_file_info(session, base_url, remote_file)

    if remote_info is None:
        # Remote file doesn't exist or couldn't be retrieved, upload it
        return True, "new file"

    # Get local file info
    local_stat = local_file.stat()
    local_size = local_stat.st_size

    # Files are the same size, assume they're identical

    # Compare file sizes first (quick check)
    if remote_info['size'] != local_size:
        return True, f"size changed ({remote_info['size']} -> {local_size})"

    # If sizes are the same, we assume files are identical
    # (Could add hash comparison here for more accuracy, but it's slower)
    return False, "unchanged"

def upload_webdav_file(session, base_url, local_file, remote_file, max_retries=3):
    """Upload a single file via WebDAV with retry logic"""
    url = get_webdav_url(base_url, remote_file)

    # Create remote directory if needed
    remote_dir = '/'.join(remote_file.split('/')[:-1])
    if remote_dir:
        dir_url = get_webdav_url(base_url, remote_dir)
        try:
            session.request('MKCOL', dir_url)
        except:
            pass  # Directory might already exist

    for attempt in range(max_retries):
        try:
            with open(local_file, 'rb') as f:
                response = session.put(url, data=f, timeout=120)

            if response.status_code in [200, 201, 204]:
                return True
            else:
                print(f"  Error uploading {local_file.name}: {response.status_code}")
                if attempt < max_retries - 1:
                    print(f"    Retrying in 2 seconds... (attempt {attempt + 2}/{max_retries})")
                    time.sleep(2)
                continue
        except Exception as e:
            print(f"  Error uploading {local_file.name}: {e}")
            if attempt < max_retries - 1:
                print(f"    Retrying in 2 seconds... (attempt {attempt + 2}/{max_retries})")
                time.sleep(2)
            continue

    print(f"  Failed to upload {local_file.name} after {max_retries} attempts")
    return False

def download_directory_recursive(session, base_url, remote_path, local_path):
    """Recursively download a directory and all its contents"""
    # List files and directories
    files, directories = list_webdav_directory(session, base_url, remote_path)

    # Create local directory
    local_path.mkdir(parents=True, exist_ok=True)

    success_count = 0
    total_files = 0

    # Download files in current directory
    for filename in files:
        print(f"  Downloading {filename}")
        remote_file = f"{remote_path}/{filename}"
        local_file = local_path / filename

        if download_webdav_file(session, base_url, remote_file, local_file):
            success_count += 1
        total_files += 1

        # Small delay between downloads to avoid overwhelming the server
        time.sleep(0.5)

    # Recursively download subdirectories
    for dirname in directories:
        print(f"  Entering directory: {dirname}")
        sub_remote_path = f"{remote_path}/{dirname}"
        sub_local_path = local_path / dirname

        sub_success, sub_total = download_directory_recursive(session, base_url, sub_remote_path, sub_local_path)
        success_count += sub_success
        total_files += sub_total

    return success_count, total_files

def download_from_pcloud():
    """Download thetawave/data and thetawave/media to assets/"""
    session, base_url = get_webdav_session()
    project_root = Path(__file__).parent.parent.parent
    assets_dir = project_root / "assets"
    assets_dir.mkdir(exist_ok=True)

    # Download data and media directories recursively
    for remote_dir in ["data", "media"]:
        remote_path = f"/thetawave/{remote_dir}"
        local_path = assets_dir / remote_dir

        print(f"Downloading {remote_path} -> {local_path} (recursive)")

        success_count, total_files = download_directory_recursive(session, base_url, remote_path, local_path)

        if total_files > 0:
            print(f"  Downloaded {success_count}/{total_files} files from {remote_dir}")
        else:
            print(f"  No files found in {remote_path}")

    print("Download completed")

def upload_directory_recursive(session, base_url, local_path, remote_path, execute=False):
    """Recursively upload a directory and all its contents (only new/changed files)"""
    success_count = 0
    total_files = 0
    skipped_count = 0

    # Get all files and directories
    for item in local_path.iterdir():
        if item.is_file():
            total_files += 1
            remote_file = f"{remote_path}/{item.name}"

            # Check if file needs to be uploaded
            should_upload, reason = should_upload_file(session, base_url, item, remote_file)

            if should_upload:
                if execute:
                    print(f"  Uploading {item.name} ({reason})")
                    if upload_webdav_file(session, base_url, item, remote_file):
                        success_count += 1
                else:
                    print(f"  Would upload: {item.name} ({reason})")
                    success_count += 1
            else:
                skipped_count += 1

        elif item.is_dir():
            # Recursively handle subdirectories
            sub_remote_path = f"{remote_path}/{item.name}"

            sub_success, sub_total, sub_skipped = upload_directory_recursive(
                session, base_url, item, sub_remote_path, execute
            )

            # Only show directory entry if it contains files to upload
            if sub_success > 0:
                if execute:
                    print(f"  Entering directory: {item.name}")
                else:
                    print(f"  Would enter directory: {item.name}")

            success_count += sub_success
            total_files += sub_total
            skipped_count += sub_skipped

    return success_count, total_files, skipped_count

def upload_to_pcloud(execute=False):
    """Upload changed/added files from assets/ to pCloud"""
    if not execute:
        print("DRY RUN MODE - No files will be uploaded")
        print("Add --execute to actually upload files")
        print()

    session, base_url = get_webdav_session()
    project_root = Path(__file__).parent.parent.parent
    assets_dir = project_root / "assets"

    if not assets_dir.exists():
        print("Error: assets/ directory not found")
        return

    total_files = 0
    total_success = 0
    total_skipped = 0

    # Upload data and media directories recursively
    for local_dir in ["data", "media"]:
        local_path = assets_dir / local_dir
        remote_path = f"/thetawave/{local_dir}"

        if not local_path.exists():
            print(f"Skipping {local_dir} - directory not found")
            continue

        if execute:
            print(f"Checking {local_path} -> {remote_path} (recursive)")
        else:
            print(f"Would check {local_path} -> {remote_path} (recursive)")

        success_count, file_count, skipped_count = upload_directory_recursive(
            session, base_url, local_path, remote_path, execute
        )

        total_files += file_count
        total_success += success_count
        total_skipped += skipped_count

        if execute:
            if success_count > 0:
                print(f"  Uploaded {success_count} files from {local_dir}")
        else:
            if success_count > 0:
                print(f"  Would upload {success_count} files from {local_dir}")
            else:
                print(f"  No changes in {local_dir}")

    if execute:
        if total_success > 0:
            print(f"Upload completed: {total_success} files uploaded")
        else:
            print("No files needed uploading - all files are up to date")
    else:
        print(f"\nDRY RUN SUMMARY:")
        if total_success > 0:
            print(f"Files that would be uploaded: {total_success}")
        if total_skipped > 0:
            print(f"Files that would be skipped (unchanged): {total_skipped}")
        if total_success > 0:
            print(f"To actually upload these files, run:")
            print(f"./sync.sh upload --execute")
        else:
            print("No files need uploading - all files are already up to date")

def test_pcloud_connection():
    """Test pCloud connection and list directory contents"""
    session, base_url = get_webdav_session()

    print("Testing pCloud connection...")

    # Test root directory
    files, dirs = list_webdav_directory(session, base_url, "/thetawave")
    print(f"Files in /thetawave: {files}")
    print(f"Directories in /thetawave: {dirs}")

    # Test data directory
    files, dirs = list_webdav_directory(session, base_url, "/thetawave/data")
    print(f"Files in /thetawave/data: {files}")
    print(f"Directories in /thetawave/data: {dirs}")

    # Test media directory
    files, dirs = list_webdav_directory(session, base_url, "/thetawave/media")
    print(f"Files in /thetawave/media: {files}")
    print(f"Directories in /thetawave/media: {dirs}")

def main():
    if len(sys.argv) < 2:
        print(__doc__)
        sys.exit(1)

    command = sys.argv[1]

    if command == 'download':
        download_from_pcloud()
    elif command == 'upload':
        execute = len(sys.argv) > 2 and sys.argv[2] == '--execute'
        upload_to_pcloud(execute)
    elif command == 'test':
        test_pcloud_connection()
    else:
        print(f"Unknown command: {command}")
        print(__doc__)
        sys.exit(1)

if __name__ == '__main__':
    main()
