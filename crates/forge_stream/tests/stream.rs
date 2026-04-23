/// Integration test for forge_stream
use forge_stream::MpscStream;
use futures::StreamExt;

#[tokio::test]
fn mpsc_channel_sends_and_receives() {
    // Create an MpscStream that sends a value
    let mut stream = MpscStream::spawn(|tx| async move {
        let _ = tx.send("hello").await;
    });

    let value = futures::block_on(stream.next());
    assert_eq!(value, Some("hello"));
}

#[tokio::test]
fn mpsc_stream_sends_multiple_values() {
    let mut stream = MpscStream::spawn(|tx| async move {
        let _ = tx.send(42i32).await;
        let _ = tx.send(99i32).await;
    });

    let first = futures::block_on(stream.next());
    let second = futures::block_on(stream.next());

    assert_eq!(first, Some(42));
    assert_eq!(second, Some(99));
}
