use std::env;

use axum::Router;
use axum::body::Body;
use axum::http::Request;

pub(crate) struct Config {
    pub dsn: Option<String>,
}

impl Config {
    pub(crate) fn from_env() -> Config {
        Config {
            dsn: env::var("SENTRY_DSN").ok(),
        }
    }
}

pub(crate) fn init(chers_config: &super::Config) -> Option<::sentry::ClientInitGuard> {
    let sentry_config = Config::from_env();
    let Some(dsn) = sentry_config.dsn else {
        return None;
    };

    Some(::sentry::init((
        dsn,
        ::sentry::ClientOptions {
            release: Some(
                format!(
                    "{}@{}",
                    chers_config.service_name, chers_config.service_version
                )
                .into(),
            ),
            environment: Some(chers_config.environment.to_owned().into()),
            traces_sample_rate: 1.0,
            ..Default::default()
        },
    )))
}

pub(crate) fn apply_middleware(app: Router) -> Router {
    use ::sentry::integrations::tower::{NewSentryLayer, SentryHttpLayer};

    app.layer(SentryHttpLayer::new().enable_transaction())
        .layer(NewSentryLayer::<Request<Body>>::new_from_top())
}
