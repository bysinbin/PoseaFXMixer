fn main() {
    #[cfg(windows)]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("assets/icon.ico");
        res.set("ProductName", "Posea Audio");
        res.set("FileDescription", "Posea Audio Mixer & Enhancer");
        res.set("LegalCopyright", "Copyright (c) 2026 Posea Audio");
        res.set("OriginalFilename", "posea_fx_mixer.exe");
        if let Err(e) = res.compile() {
            eprintln!("Warning: Failed to compile Windows resource: {}", e);
        }
    }
}
