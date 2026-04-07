# devtool.ps1 — Run the devtool utility.
$ErrorActionPreference = "Stop"
Set-Location $PSScriptRoot

cargo run -p devtool -- @args
