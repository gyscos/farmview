extern crate slack_hook;

use self::slack_hook::{PayloadBuilder, Slack};

pub fn send_alert(
    token: &str,
    channel: &str,
    host: &str,
    disk: &str,
    mount: &str,
    usage: usize,
) -> self::slack_hook::Result<()> {
    let slack = Slack::new(token)?;

    let p = PayloadBuilder::new()
        .text(format!(
            "`{}` (`{}`) on `{}` is getting critical at {}% usage.",
            mount,
            disk,
            host,
            usage
        ))
        .channel(channel)
        .username("FarmView")
        .build()?;

    slack.send(&p)
}
