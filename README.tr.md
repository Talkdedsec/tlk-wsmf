# Who Stole My Focus

Yazıyorsunuz. İstemediğiniz bir pencere öne fırlıyor ve sonraki birkaç tuşu o yiyor.
Fark ettiğinizde cümlenin yarısı başka bir yere gitmiş oluyor.

Bu, iki soruya cevap veren küçük bir Windows tray programı: **bunu hangi uygulama
yaptı**, ve isterseniz **bir daha yapmasın**.

[English](README.md)

## Neden var

Windows'un buna karşı zaten bir savunması var. Foreground lock timeout ayarı, arka
plandaki uygulamaların canı istediğinde `SetForegroundWindow` çağırmasını engellemek
için. Bir sürü kurulum programı bu değeri sessizce `0` yapıyor; ondan sonra her şey
her şeyi bölebiliyor.

Bu özellik Microsoft'un kendi PowerToys deposunda 2019'dan beri isteniyor ve hâlâ
yok. Ayarı kontrol eden registry anahtarının arayüzü yok. İnsanların bulduğu tek
üçüncü parti araç ya terk edilmiş ya da Defender tarafından işaretleniyor.

## Ne yapıyor

Tray ikonundan değişen üç mod:

- **Watch only** (varsayılan) — pencerelerinize hiç dokunmaz. Odağı alan her
  uygulamayı, zamanını ve pencere başlığını kaydeder. Buradan başlayın.
- **Guard** — siz yazarken alınan odağı geri verir, ve engel listenizdeki
  uygulamalardan her durumda geri alır.
- **Strict** — izin listesinde olmayan her şeyden odağı geri alır.

Odağı geri aldığında, araya giren pencereyi görev çubuğunda yanıp söndürür; Windows
kendi engellediğinde ne yapıyorsa aynısı. Hiçbir şey kaybolmaz, sadece sırasını
bekler.

Kayıttaki herhangi bir satıra sağ tıklayıp o uygulamayı engelleyebilir, kalıcı izin
verebilir ya da klasörünü açıp aslında ne olduğuna bakabilirsiniz.

## Kurulum

[Releases](https://github.com/Talkdedsec/tlk-wsmf/releases) sayfasından `wsmf.exe`
dosyasını indirip çalıştırın. Tek dosya, kurulum yok, ayrıca yüklenecek çalışma
ortamı yok. Tray'de belirir.

Windows ile başlaması için tray menüsünden işaretleyin. Kaldırmak için: çıkın, exe'yi
silin, kayıt ve ayarlar da gitsin isterseniz `%APPDATA%\wsmf` klasörünü silin.

## Kararı nasıl veriyor

Öne gelen her pencere hırsız değildir; çoğu zaman zaten sizsiniz, tıklamışsınız ya da
Alt+Tab yapmışsınızdır. Bunu yanlış yapmak asıl sorundan kötüdür, o yüzden şüphe
her zaman sizin lehinize:

| Durum | Ne oluyor |
|---|---|
| Son 400 ms içinde tıkladınız | Dokunulmaz |
| Şu anda bir fare düğmesi basılı | Dokunulmaz |
| Alt+Tab ya da Windows tuşu basılı | Dokunulmaz |
| Zaten içinde olduğunuz uygulamanın penceresi | Dokunulmaz |
| İzin listenizde | Dokunulmaz |
| Son 1,5 sn içinde tuşa basılmış ve yukarıdakilerin hiçbiri yok | Yazıyordunuz: bu bir hırsızlık |
| Engel listenizde | Ne yapıyor olursanız olun hırsızlık |

Bir uygulama odağı almakta ısrar ederse, on saniye içinde üç denemeden sonra o
kazanır. İki programın ön plan için kavga etmesi, sizin için tek bir terbiyesiz
programdan daha kötüdür; bu yüzden program durur ve kayda "pes etti" yazar.

## Ne yapmıyor

- Klavye hook'u kurmuyor. Sisteme son girdiden bu yana ne kadar geçtiğini ve fare
  düğmesinin basılı olup olmadığını soruyor. Hangi tuşa bastığınızı hiç görmüyor.
- Ağa çıkmıyor. Makinenizden hiçbir şey ayrılmıyor.
- Kendisi yükseltilmiş yetkiyle çalışmıyorsa, yükseltilmiş bir pencereden odağı geri
  alamaz. Windows bunu bilerek engelliyor. Böyle bir durum sessizce geçilmiyor,
  kayda "geri alınamadı" olarak yazılıyor.
- Bir pencerenin *açılmasını* engellemiyor. Klavyenin nereye gideceğine karar veriyor.

## Antivirüs

Ön plan değişikliklerini okumak ve odağı taşımak bu programın işi; aynı zamanda bazı
zararlı yazılımların da yaptığı şey, dolayısıyla sezgisel tarayıcılar ilgi
gösterebilir. Sürümler, iddia ettikleri etiketten GitHub Actions ile derleniyor ve
workflow bu depoda. Tarayıcınız yine de işaretlerse, programın tamamı bin satır
civarında Rust ve hepsini okuyabilirsiniz.

## Ayarlar

Tray menüsü çoğu kişinin ihtiyacını karşılıyor. Gerisi
`%APPDATA%\wsmf\config.toml` dosyasında:

```toml
mode = "watch"              # watch, guard, strict
typing_window_ms = 1500     # bu kadar yakın bir tuş vuruşu "yazıyordu" demek
click_grace_ms = 400        # bu kadar yakın bir tıklama "kendisi açtı" demek
blocklist = []              # exe adları, küçük harf, yolsuz
allowlist = ["explorer.exe"]
flash_thief = true          # araya giren pencereyi görev çubuğunda yanıp söndür
max_restores = 3            # bu kadar denemeden sonra pes et
restore_window_secs = 10
log_to_file = true          # %APPDATA%\wsmf\focus.log
record_everything = false   # dokunulmayanları ve gerekçesini de kaydet
```

Bir karara katılmadığınızda açacağınız ayar `record_everything`: her odak
değişikliğini, neden izin verildiği gerekçesiyle birlikte yazar.

## Derleme

```
cargo build --release
```

Rust 1.85 veya üstü, MSVC toolchain. Başka bağımlılık yok. Sonuç
`target\release\wsmf.exe` yolunda tek bir 420 KB'lık çalıştırılabilir dosya.

`cargo test` yukarıdaki karar tablosunu birim testi olarak çalıştırır.

## Lisans

MIT.
