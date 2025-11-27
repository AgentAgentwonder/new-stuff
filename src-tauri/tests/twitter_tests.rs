#[cfg(test)]
mod tests {
    use app_lib::notifications::twitter::{TweetStatus, TwitterManager};

    #[test]
    fn test_tweet_status() {
        assert_eq!(TweetStatus::Pending.as_str(), "pending");
        assert_eq!(TweetStatus::Posted.as_str(), "posted");
        assert_eq!(TweetStatus::Failed.as_str(), "failed");

        assert_eq!(TweetStatus::from_str("pending"), Some(TweetStatus::Pending));
        assert_eq!(TweetStatus::from_str("unknown"), None);
    }
}
