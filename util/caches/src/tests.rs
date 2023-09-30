use super::Caches;

#[test]
fn storing() {
    let caches: Caches<String> = Caches::new();
    let first_ptr = caches.get("1".to_string());

    assert_eq!(first_ptr, "1");
    assert_eq!(
        caches.get("1".to_string()) as *const String as usize,
        first_ptr as *const String as usize
    );
}

#[test]
#[cfg(feature = "arc")]
fn arc_storing() {
    use super::arc::Caches;

    let caches: Caches<String> = Caches::new();
    let first_ptr = caches.get("1".to_string());

    assert_eq!(first_ptr.deref(), "1");
    assert_eq!(
        caches.get("1".to_string()).deref() as *const String as usize,
        first_ptr.deref() as *const String as usize
    );
}
