#![windows_subsystem = "windows"]

fn main() {
    if std::env::args().any(|arg| arg == "--panel") {
        let _ = tlk_wsmf::panel::run();
        return;
    }
    tlk_wsmf::run_guard();
}
