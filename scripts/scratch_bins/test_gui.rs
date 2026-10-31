use eframe::egui;

fn test_viewport(ctx: &egui::Context) {
    if ctx.input(|i| i.viewport().close_requested()) {
        ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
        ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
    }
}

fn main() {
    let ctx = egui::Context::default();
    test_viewport(&ctx);
}
