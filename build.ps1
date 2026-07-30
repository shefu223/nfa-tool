$ErrorActionPreference = "Stop"
$proj = $PSScriptRoot

$cargoHome  = if ($env:CARGO_HOME)  { $env:CARGO_HOME }  else { Join-Path $env:USERPROFILE ".cargo" }
$rustupHome = if ($env:RUSTUP_HOME) { $env:RUSTUP_HOME } else { Join-Path $env:USERPROFILE ".rustup" }

$sep = [char]0x1f
$env:CARGO_ENCODED_RUSTFLAGS = @(
    "-Ctarget-feature=+crt-static",
    "--remap-path-prefix=$cargoHome=cargo",
    "--remap-path-prefix=$rustupHome=rustup",
    "--remap-path-prefix=$($env:USERPROFILE)=user",
    "--remap-path-prefix=$proj=nfa-tool"
) -join $sep

Push-Location $proj
try {
    cargo build --release
    if ($LASTEXITCODE -ne 0) { throw "cargo build failed" }

    $built = Join-Path $proj "target\release\nfa.exe"
    $bytes = [System.IO.File]::ReadAllBytes($built)
    $find = [System.Text.Encoding]::ASCII.GetBytes($proj)
    $repl = New-Object byte[] $find.Length
    [Array]::Copy([System.Text.Encoding]::ASCII.GetBytes("nfa-tool"), $repl, 8)
    for ($i = 0; $i -le $bytes.Length - $find.Length; $i++) {
        $match = $true
        for ($j = 0; $j -lt $find.Length; $j++) { if ($bytes[$i + $j] -ne $find[$j]) { $match = $false; break } }
        if ($match) { [Array]::Copy($repl, 0, $bytes, $i, $repl.Length) }
    }
    [System.IO.File]::WriteAllBytes($built, $bytes)

    Copy-Item $built (Join-Path $proj "nfa.exe") -Force
    Write-Host "Built nfa.exe (static CRT, paths scrubbed) -> $proj\nfa.exe"
} finally {
    Pop-Location
}
