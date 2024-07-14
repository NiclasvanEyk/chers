mod app;
mod handlers;
mod logging;
mod matches;

#[shuttle_runtime::main]
async fn main() -> shuttle_axum::ShuttleAxum {
    // logging::init();

    let app = app::build();

    Ok(app.into())
}
