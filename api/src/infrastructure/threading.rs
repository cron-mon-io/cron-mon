use std::future::Future;

use tokio;

pub fn run_periodically_in_background<F, Fut>(seconds: u64, func: F)
where
    F: Fn() -> Fut + Send + 'static,
    Fut: Future + Send,
{
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(seconds));
        loop {
            interval.tick().await;

            func().await;
        }
    });

    ()
}
