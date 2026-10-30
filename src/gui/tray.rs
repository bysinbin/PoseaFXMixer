use tray_icon::{
    menu::{Menu, MenuItem, MenuId, PredefinedMenuItem, MenuEvent},
    Icon, TrayIcon, TrayIconBuilder, TrayIconEvent,
};

pub struct TrayManager {
    _tray_icon: TrayIcon,
    _menu: Menu,
    _show_item: MenuItem,
    _bypass_item: MenuItem,
    _quit_item: MenuItem,
    show_item_id: MenuId,
    bypass_item_id: MenuId,
    quit_item_id: MenuId,
}

pub enum TrayAction {
    ToggleShow,
    ToggleBypass,
    Quit,
    None,
}

impl TrayManager {
    pub fn new() -> Option<Self> {
        let icon = Self::generate_posea_icon().ok()?;

        let menu = Menu::new();
        let show_item = MenuItem::new("Aç / Göster", true, None);
        let bypass_item = MenuItem::new("DSP Efektlerini Aç / Kapat", true, None);
        let quit_item = MenuItem::new("Çıkış", true, None);

        let show_item_id = show_item.id().clone();
        let bypass_item_id = bypass_item.id().clone();
        let quit_item_id = quit_item.id().clone();

        let _ = menu.append(&show_item);
        let _ = menu.append(&bypass_item);
        let _ = menu.append(&PredefinedMenuItem::separator());
        let _ = menu.append(&quit_item);

        let tray_icon = TrayIconBuilder::new()
            .with_tooltip("Posea Audio Mixer")
            .with_icon(icon)
            .with_menu(Box::new(menu.clone()))
            .with_menu_on_left_click(false)
            .build()
            .ok()?;

        Some(Self {
            _tray_icon: tray_icon,
            _menu: menu,
            _show_item: show_item,
            _bypass_item: bypass_item,
            _quit_item: quit_item,
            show_item_id,
            bypass_item_id,
            quit_item_id,
        })
    }

    /// 32x32 boyutundaki resmi Posea Audio simgesini bellekten yükler
    fn generate_posea_icon() -> Result<Icon, String> {
        let img = image::load_from_memory(include_bytes!("../../assets/icon_32.png"))
            .map_err(|e| format!("Failed to load tray icon: {}", e))?;
        let rgba = img.to_rgba8();
        let (w, h) = rgba.dimensions();
        Icon::from_rgba(rgba.into_raw(), w, h).map_err(|e| e.to_string())
    }

    pub fn check_events(&self) -> TrayAction {
        // 1. Menü tıklamalarını kontrol et
        while let Ok(event) = MenuEvent::receiver().try_recv() {
            if event.id == self.show_item_id {
                return TrayAction::ToggleShow;
            } else if event.id == self.bypass_item_id {
                return TrayAction::ToggleBypass;
            } else if event.id == self.quit_item_id {
                return TrayAction::Quit;
            }
        }

        // 2. Tepsi simgesi sol/çift tıklamasını kontrol et
        while let Ok(event) = TrayIconEvent::receiver().try_recv() {
            match event {
                TrayIconEvent::Click { button: tray_icon::MouseButton::Left, button_state: tray_icon::MouseButtonState::Up, .. } => {
                    return TrayAction::ToggleShow;
                }
                TrayIconEvent::DoubleClick { button: tray_icon::MouseButton::Left, .. } => {
                    return TrayAction::ToggleShow;
                }
                _ => {}
            }
        }

        TrayAction::None
    }
}
