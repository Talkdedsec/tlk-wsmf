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

# The panel needs the same mark as raw RGBA, since it draws its own window icon.
$raw = Join-Path $outDir "wsmf-64.rgba"
$bmp = New-Frame 64
$bytes = New-Object byte[] (64 * 64 * 4)
$i = 0
for ($y = 0; $y -lt 64; $y++) {
    for ($x = 0; $x -lt 64; $x++) {
        $c = $bmp.GetPixel($x, $y)
        $bytes[$i++] = $c.R
        $bytes[$i++] = $c.G
        $bytes[$i++] = $c.B
        $bytes[$i++] = $c.A
    }
}
$bmp.Dispose()
[System.IO.File]::WriteAllBytes($raw, $bytes)

Write-Output "wrote $Out ($($frames.Count) sizes) and $raw"

# --- the images the README and GitHub use -----------------------------------
# Same mark, same palette, drawn rather than exported from a design tool so that
# changing a colour above changes every image below.

$ink      = [System.Drawing.Color]::FromArgb(255, 230, 232, 236)
$muted    = [System.Drawing.Color]::FromArgb(255, 139, 145, 158)
$intruder = [System.Drawing.Color]::FromArgb(255, 229, 122, 110)

function Add-Mark($g, [int]$x, [int]$y, [int]$size) {
    $mark = New-Frame $size
    $g.DrawImage($mark, $x, $y, $size, $size)
    $mark.Dispose()
}

function New-Canvas([int]$w, [int]$h) {
    $bmp = New-Object System.Drawing.Bitmap($w, $h, [System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
    $g = [System.Drawing.Graphics]::FromImage($bmp)
    $g.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::AntiAlias
    $g.TextRenderingHint = [System.Drawing.Text.TextRenderingHint]::ClearTypeGridFit
    $g.Clear($backdrop)
    return @($bmp, $g)
}

# Two window outlines: the one in front holding its place, the one behind caught
# mid-push. The whole product in one picture.
function Add-Windows($g, [int]$x, [int]$y, [int]$w, [int]$h) {
    $back = New-Object System.Drawing.Pen($intruder, 3)
    $front = New-Object System.Drawing.Pen($iris, 3)
    $g.DrawRectangle($back, ($x + [int]($w * 0.18)), ($y - [int]($h * 0.10)), $w, $h)
    $g.DrawLine($back, ($x + [int]($w * 0.18)), ($y - [int]($h * 0.10) + 22), ($x + [int]($w * 0.18) + $w), ($y - [int]($h * 0.10) + 22))
    $fill = New-Object System.Drawing.SolidBrush($backdrop)
    $g.FillRectangle($fill, $x, ($y + [int]($h * 0.10)), $w, $h)
    $g.DrawRectangle($front, $x, ($y + [int]($h * 0.10)), $w, $h)
    $g.DrawLine($front, $x, ($y + [int]($h * 0.10) + 22), ($x + $w), ($y + [int]($h * 0.10) + 22))
    $caret = New-Object System.Drawing.Pen($ink, 3)
    $g.DrawLine($caret, ($x + 28), ($y + [int]($h * 0.10) + 52), ($x + 28), ($y + [int]($h * 0.10) + 86))
    $back.Dispose(); $front.Dispose(); $fill.Dispose(); $caret.Dispose()
}

# Logo on its own, for the top of the README.
$logo = New-Frame 512
$logo.Save((Join-Path $outDir "logo.png"), [System.Drawing.Imaging.ImageFormat]::Png)
$logo.Dispose()

foreach ($card in @(
    @{ File = "banner.png"; W = 1280; H = 640;  Title = 64; Sub = 26 },
    @{ File = "social.png"; W = 1200; H = 630;  Title = 62; Sub = 25 }
)) {
    $pair = New-Canvas $card.W $card.H
    $bmp = $pair[0]; $g = $pair[1]

    $markSize = [int]($card.H * 0.24)
    $left = [int]($card.W * 0.07)
    # The text block is two lines; centre the pair of them, not the first one.
    $top = [int](($card.H - $card.Title - $card.Sub - 26) / 2)
    Add-Mark $g $left $top $markSize

    $titleFont = New-Object System.Drawing.Font("Segoe UI Semibold", $card.Title, [System.Drawing.FontStyle]::Regular, [System.Drawing.GraphicsUnit]::Pixel)
    $subFont = New-Object System.Drawing.Font("Segoe UI", $card.Sub, [System.Drawing.FontStyle]::Regular, [System.Drawing.GraphicsUnit]::Pixel)
    $inkBrush = New-Object System.Drawing.SolidBrush($ink)
    $mutedBrush = New-Object System.Drawing.SolidBrush($muted)

    $textX = $left + $markSize + [int]($card.W * 0.03)
    $g.DrawString("Who Stole My Focus", $titleFont, $inkBrush, $textX, $top - 10)
    $g.DrawString("names the window that took your keyboard", $subFont, $mutedBrush, ($textX + 4), ($top + $card.Title + 10))

    $windowW = [int]($card.W * 0.15)
    $windowH = [int]($card.H * 0.30)
    Add-Windows $g ([int]($card.W * 0.76)) ([int]($card.H * 0.32)) $windowW $windowH

    $titleFont.Dispose(); $subFont.Dispose(); $inkBrush.Dispose(); $mutedBrush.Dispose()
    $g.Dispose()
    $bmp.Save((Join-Path $outDir $card.File), [System.Drawing.Imaging.ImageFormat]::Png)
    $bmp.Dispose()
    Write-Output "wrote $($card.File) ($($card.W)x$($card.H))"
}

Write-Output "wrote logo.png (512)"
