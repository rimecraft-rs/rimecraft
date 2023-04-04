use super::system_details::SystemDetails;
use chrono::{Local, Utc};
use std::{
    error::Error,
    fmt::{Display, Write},
};

pub struct CrashReport<T: Error> {
    message: String,
    cause: T,
    other_sections: Vec<CrashReportSection>,
    system_details_section: SystemDetails,
}

impl<T: Error> CrashReport<T> {
    pub fn new(message: String, cause: T) -> Self {
        Self {
            message,
            cause,
            other_sections: Vec::new(),
            system_details_section: SystemDetails::new(),
        }
    }

    pub fn get_message(&self) -> &str {
        &self.message
    }

    pub fn get_cause(&self) -> &T {
        &self.cause
    }

    fn generate_witty_comment() -> String {
        let strings = [
            "Who set us up the TNT?",
            "Everything's going to plan. No, really, that was supposed to happen.",
            "Uh... Did I do that?",
            "Oops.",
            "Why did you do that?",
            "I feel sad now :(",
            "My bad.",
            "I'm sorry, Dave.",
            "I let you down. Sorry :(",
            "On the bright side, I bought you a teddy bear!",
            "Daisy, daisy...",
            "Oh - I know what I did wrong!",
            "Hey, that tickles! Hehehe!",
            "I blame KrLite.",
            "You should try our sister game, Rimceraft!",
            "Don't be sad. I'll do better next time, I promise!",
            "Don't be sad, have a hug! <3",
            "I just don't know what went wrong :(",
            "Shall we play a game?",
            "Quite honestly, I wouldn't worry myself about that.",
            "I bet Cylons wouldn't have this problem.",
            "Sorry :(",
            "Surprise! Haha. Well, this is awkward.",
            "Would you like a cupcake?",
            "Hi. I'm Rimecraft, and I'm a crashaholic.",
            "Ooh. Shiny.",
            "This doesn't make any sense!",
            "Why is it breaking :(",
            "Don't do that.",
            "Ouch. That hurt :(",
            "You're mean.",
            "This is a token for 1 free hug. Redeem at your nearest DMHouse: [~~HUG~~]",
            "There are four lights!",
            "But it works on my machine.",
        ];
        strings
            .get(Utc::now().timestamp_nanos() as usize % strings.len())
            .unwrap_or(&"Witty comment unavailable :(")
            .to_string()
    }
}

impl<T: Error> Display for CrashReport<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("---- Rimecraft Crash Report ----\n// ")?;
        f.write_str(&Self::generate_witty_comment())?;
        f.write_str(&format!(
            "\n\nTime: {}\nDescription: {}\n\n",
            Local::now().format("%Y-%m-%d %H:%M:%S"),
            self.message
        ))?;
        std::fmt::Debug::fmt(&self.cause, f)?;
        let mut i = 0;
        while i < 87 {
            f.write_char('-')?;
            i += 1
        }
        f.write_str("\n\n")?;
        for section in &self.other_sections {
            section.fmt(f)?;
            f.write_str("\n\n")?;
        }
        self.system_details_section.fmt(f)?;
        std::fmt::Result::Ok(())
    }
}

pub struct CrashReportSection {
    title: String,
    elements: Vec<SectionElement>,
}

impl CrashReportSection {
    pub fn new(title: String) -> Self {
        Self {
            title,
            elements: Vec::new(),
        }
    }

    pub fn add(&mut self, name: String, detail: Option<impl Display>) {
        self.elements.push(SectionElement::new(name, detail))
    }
}

impl Display for CrashReportSection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(format!("-- {} --\nDetails:", self.title).as_str())?;
        for element in &self.elements {
            f.write_str("\n\t")?;
            f.write_str(element.get_name())?;
            f.write_str(": ")?;
            f.write_str(element.get_detail())?;
        }
        std::fmt::Result::Ok(())
    }
}

struct SectionElement {
    name: String,
    detail: String,
}

impl SectionElement {
    pub fn new(name: String, detail: Option<impl Display>) -> Self {
        Self {
            name,
            detail: detail
                .map(|err| format!("~~ERROR~~{}", err))
                .unwrap_or(String::from("~~NULL~~")),
        }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_detail(&self) -> &str {
        &self.detail
    }
}
