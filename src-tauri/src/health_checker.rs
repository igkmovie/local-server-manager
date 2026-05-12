use std::time::Duration;

pub fn check(url: &str) -> bool {
    let agent = ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_millis(500))
        .timeout_read(Duration::from_millis(1500))
        .build();
    match agent.get(url).call() {
        Ok(resp) => matches!(resp.status(), 200 | 204 | 301 | 302),
        Err(ureq::Error::Status(code, _)) => matches!(code, 200 | 204 | 301 | 302),
        Err(_) => false,
    }
}
