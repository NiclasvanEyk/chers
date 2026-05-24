use axum::Router;
use axum::body::Body;
use axum::http::Request;

pub(crate) fn init(dsn: &str, environment: &str, release: &str) -> ::sentry::ClientInitGuard {
    ::sentry::init((
        dsn.to_owned(),
        ::sentry::ClientOptions {
            release: Some(release.to_owned().into()),
            environment: Some(environment.to_owned().into()),
            traces_sample_rate: 1.0,
            ..Default::default()
        },
    ))
}

pub(crate) fn apply_middleware(app: Router) -> Router {
    use ::sentry::integrations::tower::{NewSentryLayer, SentryHttpLayer};

    app.layer(SentryHttpLayer::new().enable_transaction())
        .layer(NewSentryLayer::<Request<Body>>::new_from_top())
}
