#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_smtp_provider_presets() {
        use app_lib::notifications::email::SmtpProvider;

        let gmail = SmtpProvider::Gmail.preset();
        assert!(gmail.is_some());
        if let Some((server, port, _, _)) = gmail {
            assert_eq!(server, "smtp.gmail.com");
            assert_eq!(port, 587);
        }

        let custom = SmtpProvider::Custom.preset();
        assert!(custom.is_none());
    }

    #[test]
    fn test_email_status_conversion() {
        use app_lib::notifications::email::EmailStatus;

        assert_eq!(EmailStatus::Sent.as_str(), "sent");
        assert_eq!(EmailStatus::Failed.as_str(), "failed");
        assert_eq!(EmailStatus::Pending.as_str(), "pending");

        assert_eq!(EmailStatus::from_str("sent"), Some(EmailStatus::Sent));
        assert_eq!(EmailStatus::from_str("failed"), Some(EmailStatus::Failed));
        assert_eq!(EmailStatus::from_str("invalid"), None);
    }
}
