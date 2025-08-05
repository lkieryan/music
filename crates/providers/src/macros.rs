#[macro_export]
macro_rules! provider_call_with_delegate_and_timeout {
    (
        $registry:expr,
        $capability:expr,
        $provider:expr,
        $call:expr,
        $timeout_secs:expr
    ) => {{
        use tokio::time::{timeout, Duration};
        use types::errors::MoosyncError;
        match timeout(Duration::from_secs($timeout_secs), $call).await {
            Ok(Ok(v)) => Ok(v),
            Ok(Err(MoosyncError::SwitchProviders(next_key))) => {
                if let Some(np) = $registry.get(&next_key).await {
                    if $provider::supports(np.as_ref(), $capability) {
                        timeout(Duration::from_secs($timeout_secs), $call).await
                            .unwrap_or_else(|_| Err(MoosyncError::String("timeout".into())))
                    } else {
                        Err(MoosyncError::String(format!(
                            "delegated provider '{}' unavailable or unsupported",
                            next_key
                        )))
                    }
                } else {
                    Err(MoosyncError::String(format!(
                        "delegated provider '{}' not found",
                        next_key
                    )))
                }
            }
            Ok(Err(e)) => Err(e),
            Err(_) => Err(MoosyncError::String("timeout".into())),
        }
    }};
}
