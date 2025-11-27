use super::router::SharedNotificationRouter;
use crate::alerts::price_alerts::{AlertTriggerEvent, NotificationChannel};

pub async fn send_alert_notifications(
    router: SharedNotificationRouter,
    event: AlertTriggerEvent,
    channels: Vec<NotificationChannel>,
) {
    let should_send_chat = channels.iter().any(|c| {
        matches!(
            c,
            NotificationChannel::Telegram
                | NotificationChannel::Slack
                | NotificationChannel::Discord
        )
    });

    if !should_send_chat {
        return;
    }

    let router_guard = router.read().await;
    if let Err(e) = router_guard
        .send_alert_notification(
            &event.alert_id,
            &event.alert_name,
            &event.symbol,
            event.current_price,
            &event.conditions_met,
        )
        .await
    {
        eprintln!("Failed to send chat notifications: {}", e);
    }
}
