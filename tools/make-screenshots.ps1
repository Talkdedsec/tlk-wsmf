# Regenerates the README images from a sample history, in both languages.
#
# Nothing is brought to the front and no window steals your focus: the windows are
# moved off screen and asked to draw themselves with PrintWindow, so you can carry
# on working while this runs. A program that fights focus theft should not commit
# one to take its own screenshots.
#
# It never touches your real %APPDATA%: the panel is started with its own.
param(
    [string]$Exe = "$PSScriptRoot\..\target\release\wsmf.exe",
    [string]$OutDir = "$PSScriptRoot\..\assets"
)

Add-Type -AssemblyName System.Drawing
Add-Type -TypeDefinition @"
using System;
using System.Text;
using System.Runtime.InteropServices;
public class Shot {
    [StructLayout(LayoutKind.Sequential)] public struct RECT { public int Left, Top, Right, Bottom; }
    public delegate bool Proc(IntPtr h, IntPtr l);
    [DllImport("user32.dll")] public static extern bool EnumWindows(Proc p, IntPtr l);
    [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr h, out uint pid);
    [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr h);
    [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
    [DllImport("user32.dll")] public static extern bool MoveWindow(IntPtr h, int x, int y, int w, int ht, bool repaint);
    [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
    [DllImport("dwmapi.dll")] public static extern int DwmGetWindowAttribute(IntPtr h, int a, out RECT r, int s);
    [DllImport("user32.dll")] public static extern bool PrintWindow(IntPtr h, IntPtr dc, uint flags);
    [DllImport("user32.dll")] public static extern bool SetWindowPos(IntPtr h, IntPtr after, int x, int y, int w, int ht, uint flags);
    [DllImport("user32.dll")] public static extern bool SetProcessDpiAwarenessContext(IntPtr context);
    public static IntPtr Biggest(uint target) {
        IntPtr hit = IntPtr.Zero;
        int widest = 0;
        EnumWindows((h, l) => {
            uint pid; GetWindowThreadProcessId(h, out pid);
            if (pid == target && IsWindowVisible(h)) {
                RECT r; GetWindowRect(h, out r);
                int w = r.Right - r.Left;
                if (w > widest) { widest = w; hit = h; }
            }
            return true;
        }, IntPtr.Zero);
        return hit;
    }
}
"@


# Grabs one window without disturbing anything. The window is parked off screen and
# told to render itself; only if that comes back blank does it come back on screen,
# and even then it is shown without being activated.
function Grab([IntPtr]$hwnd) {
    $rect = Frame $hwnd
    $w = $rect.Right - $rect.Left

    # Move only; resizing here would catch the window mid-redraw, which is why the
    # panel takes --size instead.
    [Shot]::SetWindowPos($hwnd, [IntPtr]::Zero, -($w + 300), 0, 0, 0, 0x0015) | Out-Null
    Start-Sleep -Milliseconds 1200
    $bmp = Try-Print $hwnd
    if ($bmp) { return $bmp }

    # Some GPU drawn windows stop painting once they are off screen. Bring it back,
    # but without activating it, so the keyboard stays where it was.
    [Shot]::SetWindowPos($hwnd, [IntPtr]::Zero, 40, 30, 0, 0, 0x0015) | Out-Null
    Start-Sleep -Milliseconds 1200
    $bmp = Try-Print $hwnd
    if ($bmp) { return $bmp }

    $rect = Frame $hwnd
    $bmp = New-Object System.Drawing.Bitmap(($rect.Right - $rect.Left), ($rect.Bottom - $rect.Top))
    $g = [System.Drawing.Graphics]::FromImage($bmp)
    $g.CopyFromScreen($rect.Left, $rect.Top, 0, 0, $bmp.Size)
    $g.Dispose()
    return $bmp
}

function Frame([IntPtr]$hwnd) {
    $rect = New-Object Shot+RECT
    if ([Shot]::DwmGetWindowAttribute($hwnd, 9, [ref]$rect, 16) -ne 0) {
        [Shot]::GetWindowRect($hwnd, [ref]$rect) | Out-Null
    }
    return $rect
}

# Returns the bitmap, or $null when the window refused to draw itself.
#
# PrintWindow draws into a surface the size of GetWindowRect, which includes the
# invisible resize border; the visible edge is the DWM frame. Painting the larger
# one and then cropping to the smaller is what keeps a white strip off the bottom.
function Try-Print([IntPtr]$hwnd) {
    $outer = New-Object Shot+RECT
    [Shot]::GetWindowRect($hwnd, [ref]$outer) | Out-Null
    $w = $outer.Right - $outer.Left
    $h = $outer.Bottom - $outer.Top
    if ($w -le 0 -or $h -le 0) { return $null }

    $bmp = New-Object System.Drawing.Bitmap($w, $h)
    $g = [System.Drawing.Graphics]::FromImage($bmp)
    $dc = $g.GetHdc()
    $printed = [Shot]::PrintWindow($hwnd, $dc, 2)
    $g.ReleaseHdc($dc)
    $g.Dispose()

    if (-not $printed -or (Test-Blank $bmp)) {
        $bmp.Dispose()
        return $null
    }

    $frame = Frame $hwnd
    $crop = New-Object System.Drawing.Rectangle(
        ($frame.Left - $outer.Left),
        ($frame.Top - $outer.Top),
        ($frame.Right - $frame.Left),
        ($frame.Bottom - $frame.Top))
    if ($crop.Width -le 0 -or $crop.Height -le 0 -or
        $crop.Right -gt $bmp.Width -or $crop.Bottom -gt $bmp.Height) {
        return $bmp
    }
    $cropped = $bmp.Clone($crop, $bmp.PixelFormat)
    $bmp.Dispose()
    return $cropped
}

# A window that refused to print comes back as one flat colour.
function Test-Blank([System.Drawing.Bitmap]$bmp) {
    $first = $bmp.GetPixel(4, [int]($bmp.Height / 2))
    foreach ($x in 8, [int]($bmp.Width / 3), [int]($bmp.Width / 2), ($bmp.Width - 8)) {
        foreach ($y in [int]($bmp.Height / 3), [int]($bmp.Height / 2), ($bmp.Height - 8)) {
            if ($bmp.GetPixel($x, $y) -ne $first) { return $false }
        }
    }
    return $true
}

# Without this PowerShell is DPI unaware, GetWindowRect answers in scaled
# coordinates while DWM answers in real pixels, and every crop is wrong.
[Shot]::SetProcessDpiAwarenessContext([IntPtr](-4)) | Out-Null

$sandbox = Join-Path $env:TEMP "wsmf-shots"
if (-not (Test-Path $sandbox)) { New-Item -ItemType Directory -Path $sandbox | Out-Null }
$dataDir = Join-Path $sandbox "wsmf"
if (-not (Test-Path $dataDir)) { New-Item -ItemType Directory -Path $dataDir | Out-Null }

# A fortnight of plausible history: a few repeat offenders, quiet weekends, and the
# persistent one that eventually wins.
$apps = @(
    @{ exe = "teams.exe";     path = "C:\Program Files\Teams\teams.exe";     title = "Someone is calling you"; weight = 26 },
    @{ exe = "updater.exe";   path = "C:\Program Files\Vendor\updater.exe";  title = "Update available";       weight = 18 },
    @{ exe = "steam.exe";     path = "C:\Program Files\Steam\steam.exe";     title = "Steam";                  weight = 11 },
    @{ exe = "outlook.exe";   path = "C:\Program Files\Office\outlook.exe";  title = "New mail";               weight = 9 },
    @{ exe = "epicgames.exe"; path = "C:\Program Files\Epic\epicgames.exe";  title = "Epic Games Launcher";    weight = 7 },
    @{ exe = "mspaint.exe";   path = "C:\Windows\system32\mspaint.exe";      title = "Untitled - Paint";       weight = 4 }
)
$reasons = @("typing", "typing", "typing", "blocklist", "away")
$lines = New-Object System.Collections.Generic.List[string]
$random = New-Object System.Random 7
$hours = @(9, 10, 11, 11, 14, 15, 16, 16, 17, 21)

for ($day = 13; $day -ge 0; $day--) {
    $date = (Get-Date).Date.AddDays(-$day)
    if ($date.DayOfWeek -eq "Saturday" -or $date.DayOfWeek -eq "Sunday") {
        $perDay = $random.Next(0, 3)
    } else {
        $perDay = $random.Next(4, 11)
    }
    for ($i = 0; $i -lt $perDay; $i++) {
        $pick = $random.Next(0, 75)
        $running = 0
        $app = $apps[0]
        foreach ($candidate in $apps) {
            $running += $candidate.weight
            if ($pick -lt $running) { $app = $candidate; break }
        }
        $stamp = $date.AddHours($hours[$random.Next(0, 10)]).AddMinutes($random.Next(0, 60)).AddSeconds($random.Next(0, 60))
        $reason = $reasons[$random.Next(0, $reasons.Count)]
        if ($reason -eq "away") { $verdict = "observed" } else { $verdict = "restored" }
        $lines.Add(("{0}`t{1}`t{2}`t{3}`t{4}`t{5}" -f $stamp.ToString("yyyy-MM-ddTHH:mm:sszzz"), $verdict, $reason, $app.exe, $app.path, $app.title))
    }
}

# The one that would not give up, an hour ago.
$now = (Get-Date).AddHours(-1)
foreach ($offset in 0, 2, 5) {
    $lines.Add(("{0}`t{1}`t{2}`t{3}`t{4}`t{5}" -f $now.AddSeconds($offset).ToString("yyyy-MM-ddTHH:mm:sszzz"), "restored", "blocklist", "mspaint.exe", "C:\Windows\system32\mspaint.exe", "Untitled - Paint"))
}
$lines.Add(("{0}`t{1}`t{2}`t{3}`t{4}`t{5}" -f $now.AddSeconds(7).ToString("yyyy-MM-ddTHH:mm:sszzz"), "gave_up", "persistent", "mspaint.exe", "C:\Windows\system32\mspaint.exe", "Untitled - Paint"))

Set-Content -Path (Join-Path $dataDir "focus.log") -Value ($lines | Sort-Object) -Encoding utf8

# Inner size the panel opens at, chosen per tab so nothing is cut off and no image
# ends in empty background. Windows scales these by the display DPI.
$tabSize = @{
    activity = @(1000, 640)
    stats    = @(1000, 760)
    settings = @(1060, 700)
    rules    = @(1000, 430)
}

function Capture([string]$language, [string]$tab, [string]$file) {
    $config = @"
mode = "guard"
language = "$language"
typing_window_ms = 1500
click_grace_ms = 400
blocklist = ["mspaint.exe", "updater.exe"]
allowlist = ["explorer.exe", "searchhost.exe", "wsmf.exe"]
flash_thief = true
max_restores = 3
restore_window_secs = 10
log_to_file = true
record_everything = false
start_with_windows = false
foreground_lock_timeout_ms = 200000
"@
    Set-Content -Path (Join-Path $dataDir "config.toml") -Value $config -Encoding utf8

    $previous = $env:APPDATA
    $env:APPDATA = $sandbox
    $size = $tabSize[$tab]
    $process = Start-Process $Exe -ArgumentList "--panel", "--tab", $tab, "--size", "$($size[0])x$($size[1])" -PassThru
    $env:APPDATA = $previous
    Start-Sleep -Seconds 3

    $hwnd = [Shot]::Biggest([uint32]$process.Id)
    if ($hwnd -eq [IntPtr]::Zero) {
        Write-Output "  no window for $file"
        Stop-Process -Id $process.Id -Force -ErrorAction SilentlyContinue
        return
    }
    $bmp = Grab $hwnd
    $bmp.Save((Join-Path $OutDir $file), [System.Drawing.Imaging.ImageFormat]::Png)
    $w = $bmp.Width; $h = $bmp.Height
    $bmp.Dispose()
    Stop-Process -Id $process.Id -Force -ErrorAction SilentlyContinue
    Start-Sleep -Milliseconds 400
    Write-Output "  $file  ${w}x${h}"
}

Write-Output "English:"
Capture "english" "activity" "panel-activity.png"
Capture "english" "stats"    "panel-stats.png"
Capture "english" "settings" "panel-settings.png"
Capture "english" "rules"    "panel-rules.png"

Write-Output "Turkce:"
Capture "turkish" "activity" "panel-activity.tr.png"
Capture "turkish" "stats"    "panel-stats.tr.png"
Capture "turkish" "settings" "panel-settings.tr.png"
Capture "turkish" "rules"    "panel-rules.tr.png"

# The quick view lives in the guard process, and opens when a second copy is started.
function CaptureQuickView([string]$language, [string]$file) {
    $config = @"
mode = "guard"
language = "$language"
typing_window_ms = 1500
click_grace_ms = 400
blocklist = ["mspaint.exe"]
allowlist = ["explorer.exe", "searchhost.exe", "wsmf.exe"]
flash_thief = true
max_restores = 3
restore_window_secs = 10
log_to_file = true
record_everything = false
start_with_windows = false
foreground_lock_timeout_ms = 200000
"@
    Set-Content -Path (Join-Path $dataDir "config.toml") -Value $config -Encoding utf8

    $previous = $env:APPDATA
    $previousInstance = $env:WSMF_INSTANCE
    $env:APPDATA = $sandbox
    $env:WSMF_INSTANCE = "shots"
    $guard = Start-Process $Exe -PassThru
    Start-Sleep -Seconds 2

    # Enough real events for the list to look like a list. Each application is left
    # alone for longer than the settling window, so each one becomes its own entry.
    # Classic desktop applications only: a UWP app is hosted by ApplicationFrameHost,
    # which does not stop by its own name and leaves a window behind. Several
    # different ones, so the list shows more than one name and more than one verdict:
    # mspaint is on the block list, the rest are simply recorded.
    foreach ($app in @("mspaint", "cmd", "charmap", "mspaint", "cmd")) {
        Start-Process $app -WindowStyle Normal -ErrorAction SilentlyContinue
        Start-Sleep -Milliseconds 1500
        Stop-Process -Name $app -Force -ErrorAction SilentlyContinue
        Start-Sleep -Milliseconds 1000
    }
    # A second launch is how the quick view is asked for; it exits straight after.
    $waker = Start-Process $Exe -PassThru
    Start-Sleep -Seconds 2
    if (-not $waker.HasExited) { Stop-Process -Id $waker.Id -Force -ErrorAction SilentlyContinue }
    $env:APPDATA = $previous
    $env:WSMF_INSTANCE = $previousInstance

    $hwnd = [Shot]::Biggest([uint32]$guard.Id)
    if ($hwnd -eq [IntPtr]::Zero) {
        Write-Output "  no quick view for $file (guard alive: $(-not $guard.HasExited))"
        Stop-Process -Id $guard.Id -Force -ErrorAction SilentlyContinue
        return
    }
    $bmp = Grab $hwnd
    $bmp.Save((Join-Path $OutDir $file), [System.Drawing.Imaging.ImageFormat]::Png)
    $w = $bmp.Width; $h = $bmp.Height
    $bmp.Dispose()
    Stop-Process -Id $guard.Id -Force -ErrorAction SilentlyContinue
    Start-Sleep -Milliseconds 700
    Write-Output "  $file  ${w}x${h}"
}

# A copy left behind from an earlier run holds the mutex and every guard started
# after it exits immediately.
Get-Process wsmf -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
Start-Sleep -Milliseconds 500

Write-Output "Quick view:"
CaptureQuickView "english" "quick-view.png"
CaptureQuickView "turkish" "quick-view.tr.png"
