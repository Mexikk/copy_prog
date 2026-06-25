fn main() {
    // Windows: встраиваем иконку в .exe
    #[cfg(target_os = "windows")]
    {
        if std::path::Path::new("icon.ico").exists() {
            let mut res = winres::WindowsResource::new();
            res.set_icon("icon.ico");
            res.set("ProductName", "Copy_prog");
            res.set("FileDescription", "Утилита для сбора файлов из монорепозитория");
            res.set("LegalCopyright", "Copyright © 2026 M. Mikh. Mir");
            res.set("ProductVersion", "1.0.0");
            res.set("FileVersion", "1.0.0");
            if let Err(e) = res.compile() {
                eprintln!("Warning: Failed to compile Windows resources: {}", e);
            }
        } else {
            println!("cargo:warning=Файл icon.ico не найден, сборка без иконки");
        }
    }
    
    // Linux/macOS: иконки будут упакованы через cargo-bundle и GitHub Actions
}