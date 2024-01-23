use std::sync::Arc;

use crate::DefaultEvent;

use super::Event;

#[test]
fn registering_invoking() {
    let mut event: DefaultEvent<dyn Fn(&str) -> bool + Send + Sync> = Event::new(|listeners| {
        Arc::new(move |string| {
            for listener in &listeners {
                if !listener(string) {
                    return false;
                }
            }
            true
        })
    });

    assert!(event.invoker()(
        "minecraft by mojang is a propritary software."
    ));

    register!(
        event,
        Arc::new(|string: &str| { !string.to_lowercase().contains("propritary software") })
    );
    register!(
        event,
        Arc::new(|string: &str| !string.to_lowercase().contains("mojang"))
    );
    register!(
        event,
        Arc::new(|string| { !string.to_lowercase().contains("minecraft") })
    );

    assert!(!event.invoker()(
        "minecraft by mojang is a propritary software."
    ));

    assert!(event.invoker()("i love krlite."));

    register!(
        event,
        Arc::new(|string| !string.to_lowercase().contains("krlite"))
    );

    assert!(!event.invoker()("i love krlite."));
}

#[test]
fn phases() {
    let mut event: DefaultEvent<dyn Fn(&mut String) + Send + Sync> = Event::new(|listeners| {
        Arc::new(move |string| {
            for listener in &listeners {
                listener(string);
            }
        })
    });

    register!(event, Arc::new(|string| string.push_str("genshin impact ")));
    register!(
        event,
        Arc::new(|string| string.push_str("you're right, ")),
        -3,
    );
    register!(event, Arc::new(|string| string.push_str("but ")), -2);
    register!(event, Arc::new(|string| string.push_str("is a...")), 10);

    {
        let mut string = String::new();
        event.invoker()(&mut string);
        assert_eq!(string, "you're right, but genshin impact is a...");
    }

    register!(
        event,
        Arc::new(|string| string.push_str("genshin impact, bootstrap! ")),
        -100,
    );

    {
        let mut string = String::new();
        event.invoker()(&mut string);
        assert_eq!(
            string,
            "genshin impact, bootstrap! you're right, but genshin impact is a..."
        );
    }
}
