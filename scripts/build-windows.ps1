# SAGE Windows Build Script
# Run in PowerShell: .\scripts\build-windows.ps1

Write-Host "Building SAGE for Windows..."

# Install Rust if needed
if (!(Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Host "Installing Rust..."
    Invoke-WebRequest https://win.rustup.rs -OutFile rustup-init.exe
    .\rustup-init.exe -y
    Remove-Item rustup-init.exe
}

# Build
cargo build --release --bin sage-cli

if ($LASTEXITCODE -eq 0) {
    Write-Host "✅ Build successful!"
    Write-Host "Binary: target\release\sage-cli.exe"
    
    # Create installer
    New-Item -ItemType Directory -Force -Path "dist" | Out-Null
    Copy-Item "target\release\sage-cli.exe" "dist\sage.exe"
    
    Write-Host "📦 Installer ready in dist\ folder"
} else {
    Write-Host "❌ Build failed"
}
