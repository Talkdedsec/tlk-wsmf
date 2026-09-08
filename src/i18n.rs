//! Two languages, held in one struct so the compiler refuses a missing translation.

use serde::{Deserialize, Serialize};
use windows::Win32::Globalization::GetUserDefaultUILanguage;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    /// Follow Windows: Turkish if the interface language is Turkish, English otherwise.
    #[default]
    System,
    English,
    Turkish,
}

impl Language {
    pub fn strings(self) -> &'static Strings {
        match self.resolve() {
            Language::Turkish => &TR,
            _ => &EN,
        }
    }

    fn resolve(self) -> Language {
        match self {
            Language::System => {
                const LANG_TURKISH: u16 = 0x1F;
                let primary = unsafe { GetUserDefaultUILanguage() } & 0x3FF;
                if primary == LANG_TURKISH {
                    Language::Turkish
                } else {
                    Language::English
                }
            }
            other => other,
        }
    }
}

pub struct Strings {
    pub app_name: &'static str,
    pub tagline: &'static str,

    pub mode_watch: &'static str,
    pub mode_guard: &'static str,
    pub mode_strict: &'static str,
    pub mode_watch_hint: &'static str,
    pub mode_guard_hint: &'static str,
    pub mode_strict_hint: &'static str,

    pub verdict_restored: &'static str,
    pub verdict_observed: &'static str,
    pub verdict_allowed: &'static str,
    pub verdict_gave_up: &'static str,

    pub reason_typing: &'static str,
    pub reason_blocklist: &'static str,
    pub reason_away: &'static str,
    pub reason_clicked: &'static str,
    pub reason_button_down: &'static str,
    pub reason_switching: &'static str,
    pub reason_same_app: &'static str,
    pub reason_allowlist: &'static str,
    pub reason_system_window: &'static str,
    pub reason_no_target: &'static str,
    pub reason_persistent: &'static str,
    pub reason_failed: &'static str,

    pub tab_activity: &'static str,
    pub tab_rules: &'static str,
    pub tab_stats: &'static str,
    pub tab_settings: &'static str,

    pub column_when: &'static str,
    pub column_app: &'static str,
    pub column_what: &'static str,
    pub column_why: &'static str,
    pub column_title: &'static str,

    pub filter_all: &'static str,
    pub filter_restored: &'static str,
    pub filter_seen: &'static str,
    pub search_placeholder: &'static str,
    pub nothing_yet: &'static str,
    pub nothing_matches: &'static str,

    pub summary_today: &'static str,
    pub summary_interruptions: &'static str,
    pub summary_taken_back: &'static str,
    pub summary_worst: &'static str,
    pub summary_none: &'static str,

    pub rules_blocked: &'static str,
    pub rules_allowed: &'static str,
    pub rules_blocked_hint: &'static str,
    pub rules_allowed_hint: &'static str,
    pub rules_add: &'static str,
    pub rules_remove: &'static str,
    pub rules_empty: &'static str,
    pub rules_new_placeholder: &'static str,

    pub stats_by_app: &'static str,
    pub stats_by_hour: &'static str,
    pub stats_last_days: &'static str,
    pub stats_interruptions: &'static str,
    pub stats_no_data: &'static str,
    pub stats_daily_average: &'static str,

    pub settings_mode: &'static str,
    pub settings_timing: &'static str,
    pub settings_typing_window: &'static str,
    pub settings_click_grace: &'static str,
    pub settings_max_restores: &'static str,
    pub settings_restore_window: &'static str,
    pub settings_behaviour: &'static str,
    pub settings_flash: &'static str,
    pub settings_autostart: &'static str,
    pub settings_log_to_file: &'static str,
    pub settings_record_everything: &'static str,
    pub settings_language: &'static str,
    pub settings_language_system: &'static str,
    pub settings_windows_lock: &'static str,
    pub settings_windows_lock_on: &'static str,
    pub settings_windows_lock_off: &'static str,
    pub settings_windows_lock_hint: &'static str,
    pub settings_turn_on: &'static str,
    pub settings_turn_off: &'static str,
    pub settings_saved: &'static str,

    pub tray_open_panel: &'static str,
    pub tray_quick_view: &'static str,
    pub tray_clear: &'static str,
    pub tray_pause: &'static str,
    pub tray_resume: &'static str,
    pub tray_quit: &'static str,
    pub tray_paused: &'static str,
    pub tray_caught_none: &'static str,
    pub tray_caught_one: &'static str,
    pub tray_caught_many: &'static str,
    pub first_run_title: &'static str,
    pub first_run_body: &'static str,

    pub menu_block: &'static str,
    pub menu_allow: &'static str,
    pub menu_forget: &'static str,
    pub menu_reveal: &'static str,
    pub ms_suffix: &'static str,
    pub seconds_suffix: &'static str,
    pub times_suffix: &'static str,
}

pub static EN: Strings = Strings {
    app_name: "Who Stole My Focus",
    tagline: "Which application takes your keyboard, and when",

    mode_watch: "Watch only",
    mode_guard: "Guard",
    mode_strict: "Strict",
    mode_watch_hint: "Records who took your focus. Never touches your windows.",
    mode_guard_hint: "Takes focus back while you are typing, and from blocked applications.",
    mode_strict_hint: "Takes focus back from everything that is not allowed.",

    verdict_restored: "took back",
    verdict_observed: "seen",
    verdict_allowed: "allowed",
    verdict_gave_up: "gave up",

    reason_typing: "you were typing",
    reason_blocklist: "on the block list",
    reason_away: "you were away",
    reason_clicked: "you clicked",
    reason_button_down: "mouse button held",
    reason_switching: "you switched windows",
    reason_same_app: "same application",
    reason_allowlist: "on the allow list",
    reason_system_window: "system window",
    reason_no_target: "nothing to go back to",
    reason_persistent: "keeps grabbing focus",
    reason_failed: "could not take it back",

    tab_activity: "Activity",
    tab_rules: "Rules",
    tab_stats: "Statistics",
    tab_settings: "Settings",

    column_when: "When",
    column_app: "Application",
    column_what: "What happened",
    column_why: "Why",
    column_title: "Window title",

    filter_all: "All",
    filter_restored: "Taken back",
    filter_seen: "Seen",
    search_placeholder: "Search application or title",
    nothing_yet: "Nothing recorded yet. Leave it running and carry on working.",
    nothing_matches: "Nothing matches that.",

    summary_today: "Today",
    summary_interruptions: "interruptions",
    summary_taken_back: "taken back",
    summary_worst: "Worst offender",
    summary_none: "none",

    rules_blocked: "Always take focus back from",
    rules_allowed: "Never touch",
    rules_blocked_hint: "These lose focus whatever you are doing.",
    rules_allowed_hint: "These are left alone in every mode.",
    rules_add: "Add",
    rules_remove: "Remove",
    rules_empty: "Empty",
    rules_new_placeholder: "name.exe",

    stats_by_app: "By application",
    stats_by_hour: "By hour of day",
    stats_last_days: "Last 14 days",
    stats_interruptions: "interruptions",
    stats_no_data: "Not enough recorded yet.",
    stats_daily_average: "a day, on average",

    settings_mode: "Mode",
    settings_timing: "Timing",
    settings_typing_window: "Counts as typing within",
    settings_click_grace: "Counts as your own click within",
    settings_max_restores: "Give up after",
    settings_restore_window: "Tries counted within",
    settings_behaviour: "Behaviour",
    settings_flash: "Flash the interrupting window in the taskbar",
    settings_autostart: "Start with Windows",
    settings_log_to_file: "Keep a log file",
    settings_record_everything: "Also record what was left alone",
    settings_language: "Language",
    settings_language_system: "Follow Windows",
    settings_windows_lock: "Windows focus lock",
    settings_windows_lock_on: "on",
    settings_windows_lock_off: "off",
    settings_windows_lock_hint: "Windows can refuse the interruption itself. Installers often turn this off.",
    settings_turn_on: "Turn on",
    settings_turn_off: "Turn off",
    settings_saved: "Saved",

    tray_open_panel: "Open the panel",
    tray_quick_view: "Quick view",
    tray_clear: "Clear what has been recorded",
    tray_pause: "Pause for 15 minutes",
    tray_resume: "Resume now",
    tray_quit: "Quit",
    tray_paused: "paused",
    tray_caught_none: "nothing caught today",
    tray_caught_one: "1 caught today",
    tray_caught_many: "caught today",
    first_run_title: "Running in the tray",
    first_run_body: "Watching which application takes your focus. Right-click the tray icon to see who, or to turn guarding on.",

    menu_block: "Always take focus back from",
    menu_allow: "Never touch",
    menu_forget: "Forget the rule for",
    menu_reveal: "Show me the file",
    ms_suffix: "ms",
    seconds_suffix: "s",
    times_suffix: "tries",
};

pub static TR: Strings = Strings {
    app_name: "Who Stole My Focus",
    tagline: "Klavyeni hangi uygulama alıyor, ne zaman",

    mode_watch: "Sadece izle",
    mode_guard: "Koru",
    mode_strict: "Katı",
    mode_watch_hint: "Odağı kimin aldığını kaydeder. Pencerelerine hiç dokunmaz.",
    mode_guard_hint: "Sen yazarken alınan odağı ve engelli uygulamaları geri alır.",
    mode_strict_hint: "İzin verilmeyen her şeyden odağı geri alır.",

    verdict_restored: "geri alındı",
    verdict_observed: "görüldü",
    verdict_allowed: "dokunulmadı",
    verdict_gave_up: "pes edildi",

    reason_typing: "yazıyordun",
    reason_blocklist: "engel listesinde",
    reason_away: "başında değildin",
    reason_clicked: "sen tıkladın",
    reason_button_down: "fare düğmesi basılı",
    reason_switching: "pencere değiştirdin",
    reason_same_app: "aynı uygulama",
    reason_allowlist: "izin listesinde",
    reason_system_window: "sistem penceresi",
    reason_no_target: "dönülecek pencere yok",
    reason_persistent: "ısrarla odağı alıyor",
    reason_failed: "geri alınamadı",

    tab_activity: "Etkinlik",
    tab_rules: "Kurallar",
    tab_stats: "İstatistik",
    tab_settings: "Ayarlar",

    column_when: "Ne zaman",
    column_app: "Uygulama",
    column_what: "Ne oldu",
    column_why: "Neden",
    column_title: "Pencere başlığı",

    filter_all: "Hepsi",
    filter_restored: "Geri alınan",
    filter_seen: "Görülen",
    search_placeholder: "Uygulama ya da başlık ara",
    nothing_yet: "Henüz bir şey kaydedilmedi. Açık bıraksan yeter, sen işine bak.",
    nothing_matches: "Buna uyan bir şey yok.",

    summary_today: "Bugün",
    summary_interruptions: "kesinti",
    summary_taken_back: "geri alındı",
    summary_worst: "En çok bölen",
    summary_none: "yok",

    rules_blocked: "Şunlardan odağı hep geri al",
    rules_allowed: "Şunlara hiç dokunma",
    rules_blocked_hint: "Bunlar ne yapıyor olursan ol odağı kaybeder.",
    rules_allowed_hint: "Bunlara hiçbir modda dokunulmaz.",
    rules_add: "Ekle",
    rules_remove: "Kaldır",
    rules_empty: "Boş",
    rules_new_placeholder: "ad.exe",

    stats_by_app: "Uygulamaya göre",
    stats_by_hour: "Saate göre",
    stats_last_days: "Son 14 gün",
    stats_interruptions: "kesinti",
    stats_no_data: "Henüz yeterince kayıt yok.",
    stats_daily_average: "günlük ortalama",

    settings_mode: "Mod",
    settings_timing: "Zamanlama",
    settings_typing_window: "Şu süre içindeki tuş yazıyor sayılır",
    settings_click_grace: "Şu süre içindeki tıklama senin sayılır",
    settings_max_restores: "Şu kadar denemeden sonra pes et",
    settings_restore_window: "Denemeler şu süre içinde sayılır",
    settings_behaviour: "Davranış",
    settings_flash: "Araya giren pencere görev çubuğunda yanıp sönsün",
    settings_autostart: "Windows ile başla",
    settings_log_to_file: "Kayıt dosyası tut",
    settings_record_everything: "Dokunulmayanları da kaydet",
    settings_language: "Dil",
    settings_language_system: "Windows'u izle",
    settings_windows_lock: "Windows odak kilidi",
    settings_windows_lock_on: "açık",
    settings_windows_lock_off: "kapalı",
    settings_windows_lock_hint: "Windows kesintiyi kendi de reddedebilir. Kurulum programları bunu sık sık kapatıyor.",
    settings_turn_on: "Aç",
    settings_turn_off: "Kapat",
    settings_saved: "Kaydedildi",

    tray_open_panel: "Paneli aç",
    tray_quick_view: "Hızlı bakış",
    tray_clear: "Kaydedilenleri temizle",
    tray_pause: "15 dakika duraklat",
    tray_resume: "Şimdi devam et",
    tray_quit: "Çık",
    tray_paused: "duraklatıldı",
    tray_caught_none: "bugün bir şey yakalanmadı",
    tray_caught_one: "bugün 1 tane yakalandı",
    tray_caught_many: "bugün yakalandı",
    first_run_title: "Tray'de çalışıyor",
    first_run_body: "Odağı hangi uygulamanın aldığını izliyor. Kimin aldığını görmek ya da korumayı açmak için tray ikonuna sağ tıkla.",

    menu_block: "Şundan odağı hep geri al:",
    menu_allow: "Şuna hiç dokunma:",
    menu_forget: "Şunun kuralını unut:",
    menu_reveal: "Dosyayı göster",
    ms_suffix: "ms",
    seconds_suffix: "sn",
    times_suffix: "deneme",
};

#[cfg(test)]
mod tests {
    use super::*;

    /// The struct guarantees every key exists in both languages. This catches the
    /// other half: a key that exists but was left blank, or left in English.
    #[test]
    fn no_translation_is_empty() {
        for (name, en, tr) in pairs() {
            assert!(!en.trim().is_empty(), "English {name} is empty");
            assert!(!tr.trim().is_empty(), "Turkish {name} is empty");
        }
    }

    #[test]
    fn turkish_is_actually_translated() {
        // A handful of strings are the same on purpose: the product name and unit
        // suffixes. Everything else being identical means someone forgot.
        let same: Vec<&str> = pairs()
            .into_iter()
            .filter(|(_, en, tr)| en == tr)
            .map(|(name, _, _)| name)
            .collect();
        assert_eq!(
            same,
            vec!["app_name", "ms_suffix"],
            "unexpected untranslated strings"
        );
    }

    fn pairs() -> Vec<(&'static str, &'static str, &'static str)> {
        macro_rules! fields {
            ($($field:ident),+ $(,)?) => {
                vec![$((stringify!($field), EN.$field, TR.$field)),+]
            };
        }
        fields![
            app_name,
            tagline,
            mode_watch,
            mode_guard,
            mode_strict,
            mode_watch_hint,
            mode_guard_hint,
            mode_strict_hint,
            verdict_restored,
            verdict_observed,
            verdict_allowed,
            verdict_gave_up,
            reason_typing,
            reason_blocklist,
            reason_away,
            reason_clicked,
            reason_button_down,
            reason_switching,
            reason_same_app,
            reason_allowlist,
            reason_system_window,
            reason_no_target,
            reason_persistent,
            reason_failed,
            tab_activity,
            tab_rules,
            tab_stats,
            tab_settings,
            column_when,
            column_app,
            column_what,
            column_why,
            column_title,
            filter_all,
            filter_restored,
            filter_seen,
            search_placeholder,
            nothing_yet,
            nothing_matches,
            summary_today,
            summary_interruptions,
            summary_taken_back,
            summary_worst,
            summary_none,
            rules_blocked,
            rules_allowed,
            rules_blocked_hint,
            rules_allowed_hint,
            rules_add,
            rules_remove,
            rules_empty,
            rules_new_placeholder,
            stats_by_app,
            stats_by_hour,
            stats_last_days,
            stats_interruptions,
            stats_no_data,
            stats_daily_average,
            settings_mode,
            settings_timing,
            settings_typing_window,
            settings_click_grace,
            settings_max_restores,
            settings_restore_window,
            settings_behaviour,
            settings_flash,
            settings_autostart,
            settings_log_to_file,
            settings_record_everything,
            settings_language,
            settings_language_system,
            settings_windows_lock,
            settings_windows_lock_on,
            settings_windows_lock_off,
            settings_windows_lock_hint,
            settings_turn_on,
            settings_turn_off,
            settings_saved,
            tray_open_panel,
            tray_quick_view,
            tray_clear,
            tray_pause,
            tray_resume,
            tray_quit,
            tray_paused,
            tray_caught_none,
            tray_caught_one,
            tray_caught_many,
            first_run_title,
            first_run_body,
            menu_block,
            menu_allow,
            menu_forget,
            menu_reveal,
            ms_suffix,
            seconds_suffix,
            times_suffix,
        ]
    }
}
