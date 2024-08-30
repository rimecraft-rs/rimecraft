use crate::{pool::Pool, TestId};

#[test]
fn test_id_single_thread() {
    // Simulate a test thread
    let jh = std::thread::spawn(TestId::current);

    let current = TestId::current();
    let other = jh.join().unwrap();

    assert_ne!(current, other, "test IDs should be unique");
    assert_eq!(
        current,
        TestId::current(),
        "test ID should be the same for the same thread"
    );
}

#[test]
fn test_id_multi_thread() {
    let current = TestId::current();
    let jh = std::thread::spawn(move || {
        TestId::capture(current);
        TestId::current()
    });

    let other = jh.join().unwrap();

    assert_eq!(
        current, other,
        "test ID should be the same for the same test"
    );
}

#[test]
fn pool_same_test() {
    let pool: Pool<u64> = Pool::new();

    let current = unsafe { *pool.get_or_init(|| 1) };
    let current2 = unsafe { *pool.get_or_init(|| 2) };
    assert_eq!(current, current2, "value should be the same for one test");
}

#[test]
fn pool_diff_test() {
    let pool: Pool<u64> = Pool::new();

    let current = unsafe { *pool.get_or_init(|| 1) };
    let current2 = std::thread::scope(|s| {
        // Simulate a test thread
        s.spawn(|| unsafe { *pool.get_or_init(|| 2) })
            .join()
            .unwrap()
    });

    assert_ne!(
        current, current2,
        "value should be different for different tests"
    );
}
