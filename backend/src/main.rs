use pdf_editor_backend::app;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    app::run().await
}
