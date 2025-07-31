# pCloud Sync for Thetawave Assets

Simple script to sync specific asset directories between pCloud and your local project.

## What It Does

- **Download**: Gets `thetawave/data` and `thetawave/media` from pCloud and puts them in your local `assets/` folder (recursive)
- **Upload**: Sends only new or changed files from your local `assets/data` and `assets/media` to pCloud (recursive)

## Setup

1. Set up your credentials:
```bash
cd scripts/
cp .env.example .env
# Edit .env with your pCloud email and password
```

2. The script automatically creates a virtual environment and installs dependencies when you first run it. No manual setup needed!

## Usage

### Download assets from pCloud
```bash
./sync.sh download
```
This downloads:
- `thetawave/data` → `assets/data` (all files)
- `thetawave/media` → `assets/media` (all files and subdirectories recursively)

### Preview upload changes (safe, default)
```bash
./sync.sh upload
```
This shows you what files would be uploaded without actually doing anything. Only new or changed files are detected.

### Actually upload files
```bash
./sync.sh upload --execute
```
This uploads only new or changed files:
- `assets/data` → `thetawave/data`
- `assets/media` → `thetawave/media`

### Test connection
```bash
./sync.sh test
```
This tests your pCloud connection and lists directory contents.

## Credentials

Set these in your `.env` file:
- `PCLOUD_USERNAME` - Your pCloud email
- `PCLOUD_PASSWORD` - Your pCloud password

## Example Workflow

```bash
# Download latest assets from team
./sync.sh download

# Make changes to assets/data/ or assets/media/

# Preview what would be uploaded
./sync.sh upload

# Actually upload your changes
./sync.sh upload --execute
```

## Features

- **Smart uploads**: Only uploads new or changed files (compares file sizes)
- **Recursive**: Downloads and uploads all subdirectories
- **Retry logic**: Handles pCloud connection timeouts automatically
- **Dry run**: Safe preview mode before uploading
- **Clean output**: Shows only files being changed, not unchanged files

## How File Comparison Works

The script compares local files with remote files by:
1. Checking if the remote file exists
2. Comparing file sizes
3. Only uploading if the file is new or has a different size

This ensures you only upload what's actually changed, saving time and bandwidth.

## Troubleshooting

### Connection Issues
- The script automatically retries failed operations up to 3 times
- If you see connection timeout errors, just run the command again
- Check that your pCloud credentials are correct in the `.env` file

### Files Not Being Detected
- Make sure files are in `assets/data/` or `assets/media/`
- The script only uploads files that are new or have changed sizes
- Use `./sync.sh upload` to preview what would be uploaded

### Virtual Environment
- The script creates `scripts/venv/` automatically
- If you have issues, delete `scripts/venv/` and run the script again to recreate it

That's it! Simple asset syncing for team collaboration.