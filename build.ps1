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

# Overwrite every occurrence of $find with $repl (equal length) in-place.
function Replace-AllBytes([byte[]]$bytes, [byte[]]$find, [byte[]]$repl) {
    if ($find.Length -eq 0 -or $repl.Length -ne $find.Length) { return 0 }
    $count = 0
    for ($i = 0; $i -le $bytes.Length - $find.Length; $i++) {
        $match = $true
        for ($j = 0; $j -lt $find.Length; $j++) { if ($bytes[$i + $j] -ne $find[$j]) { $match = $false; break } }
        if ($match) { [Array]::Copy($repl, 0, $bytes, $i, $repl.Length); $count++ }
    }
    return $count
}

# Build an equal-length neutral stand-in for a path so string offsets in the
# binary stay valid. Keeps a readable "C:\Users\xxxx" shape when possible.
function Neutral-Path([string]$p) {
    if ($p.StartsWith("C:\Users\", [StringComparison]::OrdinalIgnoreCase)) {
        return "C:\Users\" + ("x" * ($p.Length - 9))
    }
    return "C:\" + ("_" * [Math]::Max(0, $p.Length - 3))
}

Push-Location $proj
try {
    cargo build --release
    if ($LASTEXITCODE -ne 0) { throw "cargo build failed" }

    $built = Join-Path $proj "target\release\nfa.exe"
    $bytes = [System.IO.File]::ReadAllBytes($built)

    # 1) The one embedded absolute project path (generate_context! bakes the
    #    crate dir in, which --remap-path-prefix cannot reach).
    $find = [System.Text.Encoding]::ASCII.GetBytes($proj)
    $repl = New-Object byte[] $find.Length
    [Array]::Copy([System.Text.Encoding]::ASCII.GetBytes("nfa-tool"), $repl, 8)
    [void](Replace-AllBytes $bytes $find $repl)

    # 2) The Windows user profile path. Some C dependencies (aws-lc-sys, the
    #    rustls crypto backend) bake absolute __FILE__ paths into their compiled
    #    objects; those are C strings, not rustc source paths, so the remap flag
    #    above never touches them and the username would otherwise ship in the
    #    exe. Scrub the profile prefix in both ASCII and UTF-16LE, equal length.
    $prof = $env:USERPROFILE
    $neutral = Neutral-Path $prof
    $scrub = 0
    $scrub += Replace-AllBytes $bytes ([System.Text.Encoding]::ASCII.GetBytes($prof))   ([System.Text.Encoding]::ASCII.GetBytes($neutral))
    $scrub += Replace-AllBytes $bytes ([System.Text.Encoding]::Unicode.GetBytes($prof)) ([System.Text.Encoding]::Unicode.GetBytes($neutral))

    [System.IO.File]::WriteAllBytes($built, $bytes)

    Copy-Item $built (Join-Path $proj "nfa.exe") -Force
    Write-Host "Built nfa.exe (static CRT, paths scrubbed, $scrub user-path strings neutralized) -> $proj\nfa.exe"
} finally {
    Pop-Location
}
