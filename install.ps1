$ErrorActionPreference = "Stop"

$REPO = "ErickJ3/faze"
$INSTALL_DIR = if ($env:FAZE_INSTALL_DIR) { $env:FAZE_INSTALL_DIR } else { "$env:LOCALAPPDATA\faze" }

function Get-LatestRelease {
    $response = Invoke-RestMethod -Uri "https://api.github.com/repos/$REPO/releases/latest"
    return $response.tag_name
}

function Main {
    Write-Host "Detecting platform..."
    $arch = $env:PROCESSOR_ARCHITECTURE

    if ($arch -ne "AMD64") {
        Write-Error "Unsupported architecture: $arch"
        exit 1
    }

    $platform = "windows-x86_64"
    Write-Host "Platform: $platform"

    Write-Host "Getting latest release..."
    $version = Get-LatestRelease
    Write-Host "Latest version: $version"

    $assetName = "faze-${platform}.exe"
    $downloadUrl = "https://github.com/$REPO/releases/download/$version/${assetName}.zip"

    Write-Host "Downloading from $downloadUrl..."
    $tempDir = New-Item -ItemType Directory -Path ([System.IO.Path]::Combine([System.IO.Path]::GetTempPath(), [System.IO.Path]::GetRandomFileName()))
    $zipPath = Join-Path $tempDir "faze.zip"

    try {
        Invoke-WebRequest -Uri $downloadUrl -OutFile $zipPath

        Write-Host "Extracting..."
        Expand-Archive -Path $zipPath -DestinationPath $tempDir -Force

        Write-Host "Installing to $INSTALL_DIR..."
        New-Item -ItemType Directory -Force -Path $INSTALL_DIR | Out-Null
        Copy-Item -Path (Join-Path $tempDir "faze.exe") -Destination (Join-Path $INSTALL_DIR "faze.exe") -Force

        Write-Host ""
        Write-Host "faze installed successfully to $INSTALL_DIR\faze.exe"
        Write-Host ""

        $currentPath = [Environment]::GetEnvironmentVariable("Path", "User")
        if ($currentPath -notlike "*$INSTALL_DIR*") {
            Write-Host "NOTE: $INSTALL_DIR is not in your PATH."
            Write-Host "Add it by running:"
            Write-Host ""
            Write-Host "  `$env:Path += `";$INSTALL_DIR`""
            Write-Host "  [Environment]::SetEnvironmentVariable('Path', `$env:Path + `";$INSTALL_DIR`", 'User')"
            Write-Host ""
            Write-Host "Or restart your terminal after running:"
            Write-Host "  [Environment]::SetEnvironmentVariable('Path', [Environment]::GetEnvironmentVariable('Path', 'User') + `";$INSTALL_DIR`", 'User')"
        } else {
            Write-Host "Run 'faze --help' to get started!"
        }
    }
    finally {
        Remove-Item -Recurse -Force $tempDir -ErrorAction SilentlyContinue
    }
}

Main
