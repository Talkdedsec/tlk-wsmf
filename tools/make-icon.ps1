# Draws assets/wsmf.ico. Run it again if the mark changes; the .ico is committed
# so a normal build never needs PowerShell.
param(
    [string]$Out = "$PSScriptRoot\..\assets\wsmf.ico"
)

Add-Type -AssemblyName System.Drawing

$sizes = @(16, 32, 48, 64, 128, 256)
$backdrop = [System.Drawing.Color]::FromArgb(255, 22, 26, 36)
$sclera   = [System.Drawing.Color]::FromArgb(255, 240, 243, 248)
$iris     = [System.Drawing.Color]::FromArgb(255, 47, 129, 247)
$pupil    = [System.Drawing.Color]::FromArgb(255, 10, 14, 22)

function New-Frame([int]$size) {
    $bmp = New-Object System.Drawing.Bitmap($size, $size, [System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
    $g = [System.Drawing.Graphics]::FromImage($bmp)
    $g.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::AntiAlias
    $g.Clear([System.Drawing.Color]::Transparent)

    $radius = [Math]::Max(2, [int]($size * 0.22))
    $path = New-Object System.Drawing.Drawing2D.GraphicsPath
    $d = $radius * 2
    $path.AddArc(0, 0, $d, $d, 180, 90)
    $path.AddArc($size - $d, 0, $d, $d, 270, 90)
    $path.AddArc($size - $d, $size - $d, $d, $d, 0, 90)
    $path.AddArc(0, $size - $d, $d, $d, 90, 90)
    $path.CloseFigure()
    $brush = New-Object System.Drawing.SolidBrush($backdrop)
    $g.FillPath($brush, $path)

    # The eye: wide enough to read at 16 px, where it collapses into a light dot.
    $eyeW = $size * 0.70
    $eyeH = $size * 0.44
    $eyeX = ($size - $eyeW) / 2
    $eyeY = ($size - $eyeH) / 2
    $brush.Color = $sclera
    $g.FillEllipse($brush, $eyeX, $eyeY, $eyeW, $eyeH)

    $irisD = $size * 0.34
    $brush.Color = $iris
    $g.FillEllipse($brush, ($size - $irisD) / 2, ($size - $irisD) / 2, $irisD, $irisD)

    $pupilD = $size * 0.16
    $brush.Color = $pupil
    $g.FillEllipse($brush, ($size - $pupilD) / 2, ($size - $pupilD) / 2, $pupilD, $pupilD)

    $brush.Dispose()
    $path.Dispose()
    $g.Dispose()
    return $bmp
}

$frames = @()
foreach ($size in $sizes) {
    $bmp = New-Frame $size
    $stream = New-Object System.IO.MemoryStream
    $bmp.Save($stream, [System.Drawing.Imaging.ImageFormat]::Png)
    $frames += [pscustomobject]@{ Size = $size; Bytes = $stream.ToArray() }
    $stream.Dispose()
    $bmp.Dispose()
}

$outDir = Split-Path -Parent $Out
if (-not (Test-Path $outDir)) { New-Item -ItemType Directory -Path $outDir | Out-Null }

$fs = [System.IO.File]::Create($Out)
$bw = New-Object System.IO.BinaryWriter($fs)
$bw.Write([uint16]0)
$bw.Write([uint16]1)
$bw.Write([uint16]$frames.Count)

$offset = 6 + (16 * $frames.Count)
foreach ($frame in $frames) {
    $dim = if ($frame.Size -ge 256) { 0 } else { $frame.Size }
    $bw.Write([byte]$dim)
    $bw.Write([byte]$dim)
    $bw.Write([byte]0)
    $bw.Write([byte]0)
    $bw.Write([uint16]1)
    $bw.Write([uint16]32)
    $bw.Write([uint32]$frame.Bytes.Length)
    $bw.Write([uint32]$offset)
    $offset += $frame.Bytes.Length
}
foreach ($frame in $frames) { $bw.Write($frame.Bytes) }
$bw.Flush()
$bw.Dispose()
$fs.Dispose()

Write-Output "wrote $Out ($($frames.Count) sizes)"
