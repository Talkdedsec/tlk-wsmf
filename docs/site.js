(() => {
  const $ = (s, r = document) => r.querySelector(s);
  const $$ = (s, r = document) => [...r.querySelectorAll(s)];
  const reduce = matchMedia('(prefers-reduced-motion: reduce)').matches;
  const root = document.documentElement;
  root.classList.add('js');

  /* ---------- language ---------- */
  const TR = {
    'skip': 'İçeriğe geç',
    'nav.demo': 'Deneyin', 'nav.decide': 'Nasıl karar veriyor', 'nav.windows': 'Pencereler', 'nav.install': 'Kurulum',
    'hero.over': 'Windows 10 · 11 / tray programı / Rust / MIT',
    'hero.h1': 'Odağımı kim çaldı?',
    'hero.lede': 'Yazıyorsunuz. İstemediğiniz bir pencere öne fırlıyor ve sonraki birkaç tuşu o yiyor. Bu tray programı size <b>bunu hangi uygulamanın yaptığını</b> söylüyor, isterseniz de <b>bir daha yapmasını engelliyor</b>.',
    'cta.dl': 'wsmf.exe indir', 'cta.try': 'Olurken izleyin',
    'demo.over': '01 — Deneyin',
    'demo.h': 'Bir cümle yazın. Bir şey sözünüzü kesecek.',
    'demo.mode': 'Mod', 'demo.ph': 'Buraya yazmaya başlayın…', 'demo.quick': 'Hızlı bakış',
    'mode.watch': 'Sadece izle', 'mode.guard': 'Koru', 'mode.strict': 'Katı',
    'note.watch': 'Sadece izle: hiçbir şeye dokunmaz, kimin yaptığını yazar. Tuşlarınız araya giren pencereye gider.',
    'note.guard': 'Koru: siz yazarken alınan odak geri verilir, araya giren pencere görev çubuğunda yanıp söner.',
    'note.strict': 'Katı: izin listesinde olmayan her şeyden odak geri alınır.',
    'st.int': 'kesinti', 'st.back': 'geri alındı', 'st.lost': 'kaybolan tuş',
    'log.empty': 'Henüz hiçbir şey odağınızı almadı.',
    'log.took': 'geri alındı', 'log.seen': 'görüldü', 'log.typing': 'yazıyordunuz', 'log.notallowed': 'izin listesinde değil', 'log.lost': 'tuş kayboldu',
    'why.over': '02 — Neden var',
    'why.p1': 'Windows’un zaten bir savunması var. Foreground lock timeout, arka plandaki uygulamaların canı istediğinde <code>SetForegroundWindow</code> çağırmasını engelliyor. Bir sürü kurulum programı bu değeri sessizce <b>0</b> yapıyor; ondan sonra her şey her şeyi bölebiliyor.',
    'why.p2': 'Bu özellik Microsoft’un kendi PowerToys deposunda 2019’dan beri isteniyor. Registry anahtarının arayüzü yok. İnsanların bulduğu tek araç ya terk edilmiş ya da Defender tarafından işaretleniyor.',
    'm.over': '03 — Tray’den ya da panelden üç mod',
    'm.watch': 'Varsayılan. Pencerelerinize hiç dokunmaz. Odağı alan her uygulamayı, zamanını ve pencere başlığını kaydeder. Buradan başlayın.',
    'm.guard': 'Siz yazarken alınan odağı geri verir; engel listenizdeki uygulamalardan her durumda geri alır.',
    'm.strict': 'İzin listesinde olmayan her şeyden odağı geri alır.',
    'm.flash': 'Odağı geri aldığında araya giren pencereyi görev çubuğunda yanıp söndürür; Windows kendi engellediğinde ne yapıyorsa aynısı. Hiçbir şey kaybolmaz, sadece sırasını bekler.',
    'd.over': '04 — Kararı nasıl veriyor',
    'd.h': 'Şüphe <em>her zaman sizin lehinize.</em>',
    'd.p': 'Öne gelen bir pencere çoğu zaman zaten sizsiniz: tıklamışsınız ya da Alt+Tab yapmışsınız. Bunu yanlış yapmak asıl sorundan kötüdür.',
    'd.1': 'Son 400 ms içinde tıkladınız', 'd.2': 'Şu anda bir fare düğmesi basılı', 'd.3': 'Alt+Tab ya da Windows tuşu basılı',
    'd.4': 'Zaten içinde olduğunuz uygulamanın penceresi', 'd.5': 'İzin listenizde', 'd.6': 'İçinde olduğunuz pencere kapandı ya da küçültüldü',
    'd.7': 'Son 1,5 sn içinde tuşa basılmış ve yukarıdakilerin hiçbiri yok', 'd.8': 'Engel listenizde',
    'v.ok': 'dokunulmaz', 'v.seen': 'kaydedilir, dönülecek yer yok', 'v.typing': 'yazıyordunuz: hırsızlık', 'v.block': 'ne yaparsanız yapın hırsızlık',
    'd.launch': 'Açılan bir uygulama birkaç milisaniyede iki üç pencere açar. İlkine cevap verilir, kalanlar yok sayılır; açılış titremeye dönüşmez.',
    'd.giveup': 'Bir uygulama odağı almakta ısrar ederse on saniyede üç denemeden sonra o kazanır. İki programın ön plan kavgası, tek bir terbiyesiz programdan kötüdür.',
    'd.tests': 'Yukarıdaki her satır bir birim testi. record_everything ayarını açın, kayıt bir kararın hangi satıra denk geldiğini söylesin.',
    'w.over': '05 — İki pencere', 'w.h': 'Biri göz atmak için, biri her şey için.',
    'w.quick': 'Hızlı bakış tray ikonuna tek tıkla açılır. Win32 listesi olduğu için anında ekranda.',
    'w.quick.alt': 'Hızlı bakış: az önce odağı alanların listesi',
    'w.panel': 'Panel kendi process’i olan, dört sekmeli bir pencere. Kapalı, meşgul ya da çökmüş olsa bile odağınızı koruyan kısım çalışmaya devam eder.',
    'w.panel.alt': 'Panelin etkinlik sekmesi',
    'w.stats': 'İstatistik: sizi kim, ne zaman bölüyor.', 'w.stats.alt': 'İstatistik sekmesi',
    'w.rules': 'Kurallar: izin listesi ve engel listesi. Türkçe ya da İngilizce, Windows’un dilini izler.', 'w.rules.alt': 'İzin ve engel listeleriyle kurallar sekmesi',
    'n.over': '06 — Ne yapmıyor',
    'n.1h': 'Klavye hook’u kurmak', 'n.1': 'Sisteme son girdiden bu yana ne kadar geçtiğini ve fare düğmesinin basılı olup olmadığını soruyor. Hangi tuşa bastığınızı hiç görmüyor.',
    'n.2h': 'Ağa çıkmak', 'n.2': 'Makinenizden hiçbir şey ayrılmıyor.',
    'n.3h': 'Yükseltilmiş pencereyle kavga', 'n.3': 'wsmf de yükseltilmiş değilse Windows bunu bilerek engelliyor. Sessizce geçilmiyor, kayda “geri alınamadı” yazılıyor.',
    'n.4h': 'Pencerenin açılmasını engellemek', 'n.4': 'Sadece klavyenin nereye gideceğine karar veriyor.',
    'i.over': '07 — Kurulum', 'i.h': 'Tek dosya. <em>Kurulum yok.</em>',
    'i.p': 'Tray’de belirir. Windows ile başlaması için tray menüsünden işaretleyin. Kaldırmak için: çıkın, exe’yi silin; kayıt ve ayarlar da gitsin isterseniz %APPDATA%\\wsmf klasörünü silin.',
    'i.or': 'ya da indirin', 'i.sum': 'indirileni doğrulayın',
    'i.av': 'Ön plan değişikliklerini okumak ve odağı taşımak bazı zararlıların da yaptığı şey; sezgisel bir tarayıcı ilgilenebilir. Sürümler etiketten GitHub Actions ile derleniyor ve programın tamamı okuyabileceğiniz yaklaşık 4 800 satır Rust.',
    'copy': 'Kopyala', 'copied': 'Kopyalandı',
    'foot.q': 'Klavyeniz sizin.', 'foot.rel': 'Sürümler', 'foot.log': 'Değişiklikler', 'foot.sec': 'Güvenlik'
  };
  const EN = {
    'hero.h1': 'Who stole my focus?',
    'note.watch': 'Watch only: touches nothing, writes down who did it. Your keystrokes go to the window that jumped in.',
    'note.guard': 'Guard: focus taken while you type is handed back, and the window that jumped in flashes in the taskbar.',
    'note.strict': 'Strict: focus is taken back from everything that is not on the allow list.',
    'log.took': 'took back', 'log.seen': 'seen', 'log.typing': 'you were typing', 'log.notallowed': 'not on the allow list', 'log.lost': 'keystrokes lost',
    'copied': 'Copied'
  };
  let lang = 'en';
  const t = k => (lang === 'tr' ? TR[k] : EN[k]) ?? '';
  const orig = new Map();
  const keep = (el, a, get) => { if (!orig.has(el)) orig.set(el, {}); const o = orig.get(el); if (!(a in o)) o[a] = get(); return o[a]; };
  function applyLang(l) {
    lang = l; root.lang = l;
    const tr = k => l === 'tr' ? TR[k] : undefined;
    $$('[data-i18n]').forEach(el => { const en = keep(el, 'text', () => el.textContent); el.textContent = tr(el.dataset.i18n) ?? en; });
    $$('[data-i18n-html]').forEach(el => { const en = keep(el, 'html', () => el.innerHTML); el.innerHTML = tr(el.dataset.i18nHtml) ?? en; });
    $$('[data-i18n-aria]').forEach(el => { const en = keep(el, 'aria', () => el.getAttribute('aria-label')); el.setAttribute('aria-label', tr(el.dataset.i18nAria) ?? en); });
    $$('[data-i18n-alt]').forEach(el => { const en = keep(el, 'alt', () => el.alt); el.alt = tr(el.dataset.i18nAlt) ?? en; });
    $$('[data-i18n-ph]').forEach(el => { const en = keep(el, 'ph', () => el.placeholder); el.placeholder = tr(el.dataset.i18nPh) ?? en; });
    $$('img[data-tr]').forEach(el => { const en = keep(el, 'src', () => el.getAttribute('src')); el.src = l === 'tr' ? el.dataset.tr : en; });
    $('.mode-note').textContent = t('note.' + mode);
    $('.typed').textContent = t('hero.h1');
    $('.h1-ghost').textContent = t('hero.h1');
    try { localStorage.setItem('wsmf-lang', l); } catch {}
  }
  $('.lang').addEventListener('click', () => { applyLang(lang === 'en' ? 'tr' : 'en'); heroDone = true; finishHero(); });
  addEventListener('scroll', () => document.body.classList.toggle('scrolled', scrollY > 8), { passive: true });

  /* ---------- hero: the headline gets its last letters stolen, then handed back ---------- */
  const typed = $('.typed'), caret = $('.caret'), thief = $('.thief'), thiefIn = $('.thief-in');
  let heroDone = false;
  const wait = ms => new Promise(r => setTimeout(r, ms));
  function finishHero() {
    typed.textContent = t('hero.h1');
    caret.classList.remove('away');
    thief.classList.remove('show', 'has-caret');
  }
  async function playHero() {
    const full = t('hero.h1'), cut = full.length - 4;
    typed.textContent = '';
    await wait(500);
    for (let i = 1; i <= cut; i++) { if (heroDone) return; typed.textContent = full.slice(0, i); await wait(55 + Math.random() * 50); }
    await wait(160);
    if (heroDone) return;
    thief.classList.add('show', 'has-caret'); caret.classList.add('away');
    thiefIn.textContent = '';
    await wait(380);
    for (let i = cut + 1; i <= full.length; i++) { if (heroDone) return; thiefIn.textContent = full.slice(cut, i); await wait(70); }
    await wait(700);
    if (heroDone) return;
    // Guard steps in: the popup drops away and the keystrokes come home
    thief.classList.remove('has-caret'); thief.classList.add('caught');
    caret.classList.remove('away');
    for (let i = cut + 1; i <= full.length; i++) { if (heroDone) return; typed.textContent = full.slice(0, i); await wait(45); }
    await wait(600);
    thief.classList.remove('show', 'caught');
    heroDone = true;
  }

  /* ---------- the demo ---------- */
  let mode = 'watch';
  const pad = $('#pad'), note = $('.note'), pop = $('.popup'), popIn = $('.pop-in'), popTitle = $('.pop-title'), popBody = $('.pop-body');
  const thiefTask = $('.thief-task'), tray = $('.tray'), log = $('.log');
  const THIEVES = [
    { exe: 'updater.exe', title: 'Update available', tTitle: 'Güncelleme var', body: 'A new version is ready to install.', tBody: 'Yeni sürüm kurulmaya hazır.' },
    { exe: 'teams.exe', title: 'Someone is calling you', tTitle: 'Biri sizi arıyor', body: 'Incoming call…', tBody: 'Gelen arama…' },
    { exe: 'steam.exe', title: 'Steam', tTitle: 'Steam', body: 'Your download has finished.', tBody: 'İndirmeniz bitti.' },
    { exe: 'outlook.exe', title: 'New mail', tTitle: 'Yeni posta', body: 'You have 3 new messages.', tBody: '3 yeni iletiniz var.' }
  ];
  const st = { since: 0, k: 0, active: null, int: 0, back: 0, lost: 0, lastKey: 0 };
  const now = () => { const d = new Date(); return String(d.getHours()).padStart(2, '0') + ':' + String(d.getMinutes()).padStart(2, '0'); };
  const stats = () => { $('#s-int').textContent = st.int; $('#s-back').textContent = st.back; $('#s-lost').textContent = st.lost; };
  function addRow(exe, what, cls, why) {
    log.querySelector('.empty')?.remove();
    const li = document.createElement('li');
    li.innerHTML = `<span class="t">${now()}</span><b>${exe}</b><span class="what ${cls}">${what}</span><span class="why">${why}</span>`;
    log.prepend(li);
    return li;
  }
  function setMode(m) {
    mode = m;
    $$('.modes [role="radio"]').forEach(b => { const on = b.dataset.mode === m; b.setAttribute('aria-checked', on); b.tabIndex = on ? 0 : -1; });
    $('.mode-note').textContent = t('note.' + m);
    if (st.active) closePop();
  }
  $$('.modes [role="radio"]').forEach((b, i, all) => {
    b.addEventListener('click', () => setMode(b.dataset.mode));
    b.addEventListener('keydown', e => {
      const d = e.key === 'ArrowRight' || e.key === 'ArrowDown' ? 1 : e.key === 'ArrowLeft' || e.key === 'ArrowUp' ? -1 : 0;
      if (!d) return;
      e.preventDefault(); const n = all[(i + d + all.length) % all.length]; setMode(n.dataset.mode); n.focus();
    });
  });

  function steal() {
    const th = THIEVES[st.int % THIEVES.length];
    st.int++; stats();
    st.active = th;
    popTitle.textContent = th.exe + ' — ' + (lang === 'tr' ? th.tTitle : th.title);
    popBody.textContent = lang === 'tr' ? th.tBody : th.body;
    popIn.value = '';
    pop.hidden = false; pop.classList.remove('back'); pop.classList.add('stolen');
    pop.style.animation = 'none'; void pop.offsetWidth; pop.style.animation = '';
    thiefTask.textContent = th.exe.replace('.exe', ''); thiefTask.hidden = false; thiefTask.classList.remove('flash');
    popIn.focus({ preventScroll: true });
    note.classList.remove('focus');
    if (mode === 'watch') {
      th.row = addRow(th.exe, t('log.seen'), 'seen', t('log.typing'));
      return;
    }
    // guard and strict: hand it back the way wsmf does, then flash the thief in the taskbar
    setTimeout(() => {
      pad.focus({ preventScroll: true });
      pop.classList.remove('stolen'); pop.classList.add('back');
      thiefTask.classList.remove('flash'); void thiefTask.offsetWidth; thiefTask.classList.add('flash');
      tray.classList.remove('ping'); void tray.offsetWidth; tray.classList.add('ping');
      st.back++; stats();
      addRow(th.exe, t('log.took'), 'took', mode === 'strict' ? t('log.notallowed') : t('log.typing'));
      setTimeout(() => { if (st.active === th) closePop(); }, 2200);
    }, reduce ? 0 : 140);
  }
  function closePop() {
    pop.hidden = true; thiefTask.hidden = true; st.active = null; st.since = 0;
  }
  pad.addEventListener('focus', () => { note.classList.add('focus'); if (st.active && mode === 'watch' && !pop.hidden) closePop(); });
  pad.addEventListener('blur', () => note.classList.remove('focus'));
  pad.addEventListener('input', e => {
    if (e.inputType && !e.inputType.startsWith('insert')) return;
    st.since++;
    if (!st.active && st.since >= 12 + (st.int % 3) * 5) steal();
  });
  popIn.addEventListener('input', () => {
    if (!st.active) return;
    st.lost++; stats();
    if (st.active.row) st.active.row.querySelector('.why').textContent = `${t('log.typing')} · ${popIn.value.length} ${t('log.lost')}`;
  });
  $('.pop-x').addEventListener('click', () => { closePop(); pad.focus(); });

  /* ---------- copy + reveals ---------- */
  $$('.copy').forEach(b => b.addEventListener('click', async () => {
    try { await navigator.clipboard.writeText(b.previousElementSibling.textContent); } catch { return; }
    b.textContent = t('copied'); b.classList.add('ok');
    setTimeout(() => { b.textContent = lang === 'tr' ? TR.copy : 'Copy'; b.classList.remove('ok'); }, 1400);
  }));
  const io = new IntersectionObserver(es => es.forEach(e => { if (e.isIntersecting) { e.target.classList.add('in'); io.unobserve(e.target); } }), { rootMargin: '0px 0px -10% 0px' });
  $$('.reveal').forEach(el => io.observe(el));

  /* ---------- start ---------- */
  let saved = null;
  try { saved = localStorage.getItem('wsmf-lang'); } catch {}
  const q = new URLSearchParams(location.search).get('lang');
  applyLang(q === 'tr' || q === 'en' ? q : saved || (/^tr\b/i.test(navigator.language) ? 'tr' : 'en'));
  setMode('watch');
  if (reduce) { heroDone = true; finishHero(); } else playHero();
})();
