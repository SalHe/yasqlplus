mod app;

fn main() -> anyhow::Result<()> {
    app::App::new()?.run()
}
