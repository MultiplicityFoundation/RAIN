use super::{
    Arc, Channel, Config, Context, DiscordChannel, Result, SendMessage, SlackChannel,
    TelegramChannel,
};

pub(super) fn build_channel_by_id(config: &Config, channel_id: &str) -> Result<Arc<dyn Channel>> {
    match channel_id.to_ascii_lowercase().as_str() {
        "telegram" => {
            let tg = config
                .channels_config
                .telegram
                .as_ref()
                .context("Telegram channel is not configured")?;
            Ok(Arc::new(TelegramChannel::new(
                tg.bot_token.clone(),
                tg.allowed_users.clone(),
                tg.mention_only,
            )))
        }
        "discord" => {
            let dc = config
                .channels_config
                .discord
                .as_ref()
                .context("Discord channel is not configured")?;
            Ok(Arc::new(DiscordChannel::new(
                dc.bot_token.clone(),
                dc.guild_id.clone(),
                dc.allowed_users.clone(),
                dc.listen_to_bots,
                dc.mention_only,
            )))
        }
        "slack" => {
            let sl = config
                .channels_config
                .slack
                .as_ref()
                .context("Slack channel is not configured")?;
            Ok(Arc::new(
                SlackChannel::new(
                    sl.bot_token.clone(),
                    sl.app_token.clone(),
                    sl.channel_id.clone(),
                    Vec::new(),
                    sl.allowed_users.clone(),
                )
                .with_workspace_dir(config.workspace_dir.clone()),
            ))
        }
        other => anyhow::bail!("Unknown channel '{other}'. Supported: telegram, discord, slack"),
    }
}

pub(super) async fn send_channel_message(
    config: &Config,
    channel_id: &str,
    recipient: &str,
    message: &str,
) -> Result<()> {
    let channel = build_channel_by_id(config, channel_id)?;
    let msg = SendMessage::new(message, recipient);
    channel
        .send(&msg)
        .await
        .with_context(|| format!("Failed to send message via {channel_id}"))?;
    println!("Message sent via {channel_id}.");
    Ok(())
}
