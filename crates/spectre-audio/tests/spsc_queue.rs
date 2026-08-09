// Author: Jeff
// Date: 2026-08-09
// Description: R3 slice 3 evidence — bounded wait-free SPSC queue behavior
// Notes: Covers the queue primitive directly, including the teardown path. A ring that forgets
//   to drop its queued elements leaks silently, so the Arc strong-count test is the real gate.

use std::sync::Arc;

use spectre_audio::spsc::bounded;

#[test]
fn capacity_is_at_least_the_request() {
    let (producer, consumer) = bounded::<u32>(100);
    assert!(producer.capacity() >= 100);
    assert_eq!(producer.capacity(), consumer.capacity());
}

#[test]
fn push_and_pop_preserve_order() {
    let (mut producer, mut consumer) = bounded::<u32>(8);
    assert!(consumer.is_empty());
    for value in 0..8 {
        producer.push(value).unwrap();
    }
    assert!(!consumer.is_empty());
    for expected in 0..8 {
        assert_eq!(consumer.pop(), Some(expected));
    }
    assert_eq!(consumer.pop(), None);
    assert!(consumer.is_empty());
}

#[test]
fn a_full_queue_hands_the_value_back() {
    let (mut producer, mut consumer) = bounded::<u32>(4);
    let capacity = producer.capacity();
    for value in 0..capacity as u32 {
        producer.push(value).unwrap();
    }
    // The rejected value is returned, never dropped inside push
    assert_eq!(producer.push(999), Err(999));

    // Draining one element makes room for exactly one more
    assert_eq!(consumer.pop(), Some(0));
    producer.push(999).unwrap();
    assert_eq!(producer.push(1_000), Err(1_000));
}

#[test]
fn indices_wrap_repeatedly_without_corruption() {
    let (mut producer, mut consumer) = bounded::<u64>(4);
    let capacity = producer.capacity() as u64;

    // Cycle far past the ring length so the mask wrap is exercised many times
    for round in 0..1_000 {
        for offset in 0..capacity {
            producer.push(round * capacity + offset).unwrap();
        }
        for offset in 0..capacity {
            assert_eq!(consumer.pop(), Some(round * capacity + offset));
        }
    }
    assert!(consumer.is_empty());
}

#[test]
fn dropping_a_nonempty_queue_drops_its_elements() {
    let witness = Arc::new(());
    {
        let (mut producer, mut consumer) = bounded::<Arc<()>>(8);
        for _ in 0..5 {
            producer.push(Arc::clone(&witness)).unwrap();
        }
        assert_eq!(Arc::strong_count(&witness), 6);

        // Pop some but not all, leaving live elements in the ring at teardown
        assert!(consumer.pop().is_some());
        assert!(consumer.pop().is_some());
        assert_eq!(Arc::strong_count(&witness), 4);
    }
    // Teardown must release the three still queued, leaving only the local handle
    assert_eq!(Arc::strong_count(&witness), 1);
}

#[test]
fn elements_queued_across_a_wrap_are_dropped_at_teardown() {
    let witness = Arc::new(());
    {
        let (mut producer, mut consumer) = bounded::<Arc<()>>(4);
        let capacity = producer.capacity();
        // Advance the indices so the live range straddles the wrap point
        for _ in 0..capacity {
            producer.push(Arc::clone(&witness)).unwrap();
        }
        for _ in 0..capacity - 1 {
            consumer.pop();
        }
        for _ in 0..capacity - 1 {
            producer.push(Arc::clone(&witness)).unwrap();
        }
        assert_eq!(Arc::strong_count(&witness), capacity + 1);
    }
    assert_eq!(Arc::strong_count(&witness), 1);
}

#[test]
fn producer_and_consumer_move_to_separate_threads() {
    let (mut producer, mut consumer) = bounded::<u64>(64);
    let total = 100_000_u64;

    let writer = std::thread::spawn(move || {
        let mut sent = 0;
        while sent < total {
            // Spin only in the test harness; the audio thread never blocks like this
            if producer.push(sent).is_ok() {
                sent += 1;
            }
        }
    });

    let reader = std::thread::spawn(move || {
        let mut expected = 0;
        while expected < total {
            if let Some(value) = consumer.pop() {
                assert_eq!(value, expected, "cross-thread FIFO order must hold");
                expected += 1;
            }
        }
        expected
    });

    writer.join().unwrap();
    assert_eq!(reader.join().unwrap(), total);
}
