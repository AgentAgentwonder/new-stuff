use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_exponential_backoff() {
    use app_lib::websocket::reconnect::ExponentialBackoff;
    use app_lib::websocket::types::BackoffConfig;

    let config = BackoffConfig {
        initial_delay: Duration::from_millis(100),
        max_delay: Duration::from_secs(2),
        max_attempts: 5,
    };

    let mut backoff = ExponentialBackoff::new(config);

    let delay1 = backoff.next_delay().expect("First delay");
    assert!(delay1.as_millis() >= 80 && delay1.as_millis() <= 120);

    let delay2 = backoff.next_delay().expect("Second delay");
    assert!(delay2.as_millis() >= 160 && delay2.as_millis() <= 240);

    let delay3 = backoff.next_delay().expect("Third delay");
    assert!(delay3.as_millis() >= 320 && delay3.as_millis() <= 480);

    assert!(backoff.attempts() == 3);

    backoff.reset();
    assert!(backoff.attempts() == 0);
}

#[tokio::test]
async fn test_message_queue() {
    use app_lib::websocket::types::{MessageQueue, PriceDelta, StreamEvent};

    let mut queue: MessageQueue<StreamEvent> = MessageQueue::with_capacity(3);

    let delta1 = PriceDelta {
        symbol: "SOL".to_string(),
        price: Some(100.0),
        change: Some(5.0),
        volume: None,
        ts: 1000,
        snapshot: false,
    };

    queue.push(StreamEvent::PriceUpdate(delta1.clone()));
    queue.push(StreamEvent::PriceUpdate(delta1.clone()));
    queue.push(StreamEvent::PriceUpdate(delta1.clone()));
    queue.push(StreamEvent::PriceUpdate(delta1.clone()));

    assert!(queue.dropped_count() == 1);

    let events = queue.drain();
    assert!(events.len() == 3);
}

#[tokio::test]
async fn test_delta_state_throttle() {
    use app_lib::websocket::types::{DeltaState, PriceSnapshot};
    use std::time::Instant;

    let mut delta_state = DeltaState::<PriceSnapshot>::new(Duration::from_millis(100));

    let now = Instant::now();
    assert!(delta_state.should_emit("SOL", now));

    delta_state.mark_emitted("SOL".to_string(), now);

    let now2 = Instant::now();
    assert!(!delta_state.should_emit("SOL", now2));

    sleep(Duration::from_millis(110)).await;
    let now3 = Instant::now();
    assert!(delta_state.should_emit("SOL", now3));
}

#[tokio::test]
async fn test_connection_state_transitions() {
    use app_lib::websocket::types::ConnectionState;

    let state1 = ConnectionState::Connecting;
    let state2 = ConnectionState::Connected;
    let state3 = ConnectionState::Disconnected;
    let state4 = ConnectionState::Failed;
    let state5 = ConnectionState::Fallback;

    assert_ne!(state1, state2);
    assert_ne!(state2, state3);
    assert_ne!(state3, state4);
    assert_ne!(state4, state5);
}

#[tokio::test]
async fn test_stream_provider_id() {
    use app_lib::websocket::types::StreamProvider;

    assert_eq!(StreamProvider::Birdeye.id(), "birdeye");
    assert_eq!(StreamProvider::Helius.id(), "helius");
}

#[tokio::test]
async fn test_price_snapshot_defaults() {
    use app_lib::websocket::types::PriceSnapshot;

    let snapshot = PriceSnapshot::default();
    assert_eq!(snapshot.price, 0.0);
    assert_eq!(snapshot.change, 0.0);
    assert_eq!(snapshot.volume, None);
    assert_eq!(snapshot.ts, 0);
}

#[tokio::test]
async fn test_price_delta_creation() {
    use app_lib::websocket::types::PriceDelta;

    let delta = PriceDelta {
        symbol: "SOL".to_string(),
        price: Some(150.5),
        change: Some(3.2),
        volume: Some(1000000.0),
        ts: 1234567890,
        snapshot: false,
    };

    assert_eq!(delta.symbol, "SOL");
    assert_eq!(delta.price, Some(150.5));
    assert_eq!(delta.change, Some(3.2));
    assert_eq!(delta.volume, Some(1000000.0));
    assert!(!delta.snapshot);
}

#[tokio::test]
async fn test_stream_subscriptions() {
    use app_lib::websocket::types::StreamSubscriptions;

    let mut subs = StreamSubscriptions::default();
    assert!(subs.prices.is_empty());
    assert!(subs.wallets.is_empty());

    subs.prices.push("SOL".to_string());
    subs.prices.push("BTC".to_string());
    subs.wallets.push("wallet1".to_string());

    assert_eq!(subs.prices.len(), 2);
    assert_eq!(subs.wallets.len(), 1);
}

#[tokio::test]
async fn test_backoff_max_attempts() {
    use app_lib::websocket::reconnect::ExponentialBackoff;
    use app_lib::websocket::types::BackoffConfig;

    let config = BackoffConfig {
        initial_delay: Duration::from_millis(10),
        max_delay: Duration::from_secs(1),
        max_attempts: 3,
    };

    let mut backoff = ExponentialBackoff::new(config);

    assert!(backoff.next_delay().is_some());
    assert!(backoff.next_delay().is_some());
    assert!(backoff.next_delay().is_some());
    assert!(backoff.next_delay().is_none());
}

#[tokio::test]
async fn test_message_queue_drain() {
    use app_lib::websocket::types::{MessageQueue, StreamEvent, StreamProvider};

    let mut queue: MessageQueue<StreamEvent> = MessageQueue::with_capacity(10);

    let error1 = StreamEvent::Error {
        provider: StreamProvider::Birdeye,
        message: "Test error".to_string(),
    };

    queue.push(error1.clone());
    queue.push(error1.clone());

    let events = queue.drain();
    assert_eq!(events.len(), 2);

    let empty = queue.drain();
    assert!(empty.is_empty());
}
