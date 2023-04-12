use std::{collections::HashMap, fmt::Display};

use sysinfo::{CpuExt, System, SystemExt};

use crate::{consts, version::GameVersion};

fn get_operation_system_format() -> String {
    let sys = System::new_all();
    format!(
        "{:?} () version {:?}",
        sys.distribution_id(),
        sys.os_version()
    )
}
pub struct SystemDetails {
    sections: HashMap<String, String>,
}

impl SystemDetails {
    pub fn new() -> Self {
        let mut obj = Self {
            sections: HashMap::new(),
        };
        let sys = System::new_all();
        obj.add_section(
            "Rimecraft Version".to_string(),
            consts::GAME_VERSION.get_name().to_owned(),
        );
        obj.add_section(
            "Rimecraft Version ID".to_string(),
            consts::GAME_VERSION.get_id().to_owned(),
        );
        obj.add_section(
            "Operation System".to_string(),
            get_operation_system_format(),
        );
        obj.add_section(
            "Memory".to_string(),
            format!(
                "{} bytes ({} MiB) / {} ({} MiB)",
                sys.free_memory(),
                sys.free_memory() / 0x100000,
                sys.total_memory(),
                sys.total_memory() / 0x100000,
            ),
        );
        obj.add_section("CPUs".to_string(), format!("{:?}", sys.cpus()));
        let cpu = sys.global_cpu_info();
        obj.add_section("Processor Vendor".to_string(), cpu.vendor_id().to_owned());
        obj.add_section("Processor Name".to_string(), cpu.name().to_owned());
        obj.add_section("Frequency (GHz)".to_string(), cpu.frequency().to_string());
        obj
    }

    pub fn add_section(&mut self, name: String, value: String) {
        self.sections.insert(name, value);
    }

    pub fn collect(&self) -> String {
        self.sections
            .iter()
            .map(|a| format!("{}: {}", a.0, a.1))
            .collect()
    }
}

impl Default for SystemDetails {
    fn default() -> Self {
        Self::new()
    }
}

impl Display for SystemDetails {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("-- System Details --\nDetails:")?;
        for o in &self.sections {
            f.write_str("\n\t")?;
            f.write_str(o.0)?;
            f.write_str(": ")?;
            f.write_str(o.1)?;
        }
        std::fmt::Result::Ok(())
    }
}
