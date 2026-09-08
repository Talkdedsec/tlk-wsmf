**Download `wsmf.exe` below and run it.** One file, no installer, nothing else to
install. It appears in the tray and starts in watch-only mode, which never touches
your windows — it just records which application took your keyboard, and when.

![The panel](https://raw.githubusercontent.com/Talkdedsec/tlk-wsmf/main/assets/panel-activity.png)

### What it is

Windows has a defence against a background application stealing your keyboard: the
foreground lock timeout. Installers routinely set it to `0`, and after that anything
can interrupt anything. The fix has been open in Microsoft's own PowerToys
repository since 2019.

This names the culprit, and can hand focus straight back to the window you were
using. Three modes, switched from the tray: **watch only** (default, records and
nothing else), **guard** (takes focus back while you are typing and from your block
list), **strict** (takes it back from everything not on your allow list).

- No keyboard hook. It asks the system how long since the last input, never which key.
- No network code. Nothing leaves your machine.
- English and Turkish, following your Windows display language.

### Verifying this download

`wsmf.exe.sha256` is beside the executable. Compare it yourself:

```powershell
(Get-FileHash wsmf.exe -Algorithm SHA256).Hash
```

The binary was built by the workflow run linked at the bottom of this page, from
this tag, in a clean runner. Nothing was uploaded by hand.

**It is not code signed yet**, so SmartScreen will warn on first run: More info →
Run anyway. Windows Defender was clean on the build machine, and the whole program
is a few thousand lines of Rust you can read in this repository.

### Removing it

Quit from the tray menu, delete the exe, and delete `%APPDATA%\wsmf` if you want the
log and settings gone too. Nothing else is written anywhere.

---

### Türkçe

`wsmf.exe` dosyasını indirip çalıştırın. Tek dosya, kurulum yok. Tray'de belirir ve
pencerelerinize hiç dokunmayan izleme modunda başlar: sadece klavyenizi hangi
uygulamanın aldığını kaydeder.

Odağı geri almasını isterseniz tray menüsünden **Koru** moduna alın; siz yazarken
araya giren pencereden odağı geri alır ve o pencereyi görev çubuğunda yanıp
söndürür, tıpkı Windows'un kendi yaptığı gibi. Hiçbir şey kaybolmaz.

Klavye hook'u kurmaz, ağa çıkmaz. Türkçe ve İngilizce; varsayılan olarak Windows'un
dilini izler.

Sürüm henüz imzalı değil, ilk çalıştırmada SmartScreen uyarır: Ek bilgi → Yine de
çalıştır. Dosyanın SHA-256 değeri yanındaki `.sha256` dosyasında; derleme bu
etiketten, bu depodaki workflow ile yapıldı.
