# Linux Development Setup for Reelix

## Required MakeMKV Components

Reelix requires MakeMKV binaries and shared libraries that are **NOT** included in the repository. To update or add MakeMKV support, you need to:

1. Install MakeMKV on a Linux system
2. Extract the binaries and shared libraries
3. Use the provided script to copy and rename them correctly

### What You Need to Copy

- **Binaries** (executables):

  - `makemkvcon` - Main MakeMKV command-line tool
  - `mmgplsrv` - GPL license server for MakeMKV
  - `mmccextr` - Closed caption extractor (may be integrated into makemkvcon)

- **Shared Libraries** (`.so` files):
  - `libdriveio.so.0` - Drive I/O library
  - `libmakemkv.so.1` - Core MakeMKV library
  - `libmmbd.so.0` - Blu-ray disc library
  - Additional dependencies MakeMKV installs

## Installation Steps

### 1. Install MakeMKV on Linux

There are too many ways to do this, I did it with flatpak but I'm sure you can do it your way. I'm tempted to not ship MKV Maker with linux and point to however the user decided to install it given how complex it can be. 

### 2. Gather MakeMKV Files

Create a temporary directory and copy all MakeMKV files:

```bash
# Create a temporary directory
mkdir -p tmp/linux

# Copy binaries
cp /usr/bin/makemkvcon tmp/linux/
cp /usr/bin/mmgplsrv tmp/linux/  # If it exists as standalone

# Copy shared libraries
cp /usr/lib/libdriveio.so.0 tmp/linux/
cp /usr/lib/libmakemkv.so.1 tmp/linux/
cp /usr/lib/libmmbd.so.0 tmp/linux/

# Check for mmccextr (may be in /usr/share/makemkv/ or integrated)
# If found as standalone:
cp /usr/bin/mmccextr tmp/linux/  # If it exists
```

### 3. Use the Rename Script

The project includes a script to automatically copy and rename files with the correct architecture suffix:

```bash
# Run the script to copy and rename all files correctly
./scripts/rename-linux-makemkv.sh -i tmp/linux

# This will:
# - Copy all .so* files to src-tauri/libraries/linux/
# - Copy and rename binaries to src-tauri/binaries/ with -x86_64-unknown-linux-gnu suffix
# - Handle makemkvcon, mmgplsrv, and mmccextr automatically
```

### 4. Verify the Files

Check that files were copied correctly:

```bash
# Check binaries
ls -lh src-tauri/binaries/*linux-gnu

# Check libraries
ls -lh src-tauri/libraries/linux/

# Ensure binaries are executable
chmod +x src-tauri/binaries/*linux-gnu
```

### 5. Rebuild your app

```bash
cargo tauri dev --config src-tauri/tauri.linux.conf.json
```

## Updating MakeMKV Versions

When a new version of MakeMKV is released:

1. Install the updated version on your Linux system
2. Gather all files again (binaries and libraries)
3. Run the rename script to update the project files
4. Test the build to ensure compatibility

**Important**: Package managers only install the MakeMKV application files. You must manually copy the binaries and shared libraries to this project, as they are not included in the repository due to licensing and size constraints.

## Temporary Workaround (Testing Only)

For testing the build process without actual ripping functionality, you can create dummy binaries:

```bash
# Create dummy binaries (these won't actually work for ripping!)
cp src-tauri/binaries/makemkvcon-x86_64-unknown-linux-gnu src-tauri/binaries/mmgplsrv-x86_64-unknown-linux-gnu
cp src-tauri/binaries/makemkvcon-x86_64-unknown-linux-gnu src-tauri/binaries/mmccextr-x86_64-unknown-linux-gnu
```

## Current Status

- ✅ `cargo tauri dev` builds and runs successfully
- ✅ FFmpeg 7 libraries installed
- ✅ All Tauri dependencies installed
- ⚠️ Missing `mmgplsrv` and `mmccextr` Linux binaries
- ⚠️ DVD ripping will fail until these are properly installed

## Error Messages You'll See

Without these binaries, you'll see:

```
Failed to execute external program 'mmgplsrv' from location '/path/to/mmgplsrv'
Failed to execute external program 'ccextractor' from location '/path/to/mmccextr'
```
