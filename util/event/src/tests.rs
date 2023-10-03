use super::Event;

#[test]
fn registering_invoking() {
    let mut event: Event<dyn Fn(&str) -> bool> = Event::new(|listeners| {
        Box::new(move |string| {
            for listener in listeners {
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

    event.register(Box::new(|string| {
        !string.to_lowercase().contains("propritary software")
    }));
    event.register(Box::new(|string| !string.to_lowercase().contains("mojang")));
    event.register(Box::new(|string| {
        !string.to_lowercase().contains("minecraft")
    }));

    assert!(!event.invoker()(
        "minecraft by mojang is a propritary software."
    ));

    assert!(event.invoker()("i love krlite."));

    event.register(Box::new(|string| !string.to_lowercase().contains("krlite")));

    assert!(!event.invoker()("i love krlite."));
}

#[test]
fn phases() {
    let mut event: Event<dyn Fn(&mut String)> = Event::new(|listeners| {
        Box::new(move |string| {
            for listener in listeners {
                listener(string);
            }
        })
    });

    event.register(Box::new(|string| string.push_str("genshin impact ")));
    event.register_with_phase(Box::new(|string| string.push_str("you're right, ")), -3);
    event.register_with_phase(Box::new(|string| string.push_str("but ")), -2);
    event.register_with_phase(Box::new(|string| string.push_str("is a...")), 10);

    {
        let mut string = String::new();
        event.invoker()(&mut string);
        assert_eq!(string, "you're right, but genshin impact is a...");
    }

    event.register_with_phase(
        Box::new(|string| string.push_str("genshin impact, bootstrap! ")),
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
