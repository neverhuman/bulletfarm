use std::collections::BTreeMap;

use crate::coord::CoordError;

#[derive(Default)]
pub(crate) struct Options {
    values: BTreeMap<String, Vec<String>>,
    flags: Vec<String>,
}

impl Options {
    pub(crate) fn parse(args: &[String]) -> Result<Self, CoordError> {
        let mut options = Self::default();
        let mut index = 0;
        while index < args.len() {
            let name = args[index].strip_prefix("--").ok_or_else(|| {
                CoordError::new("INVALID_ARGUMENT", format!("unexpected {}", args[index]))
            })?;
            if matches!(name, "json" | "all") {
                if options.flags.iter().any(|flag| flag == name) {
                    return Err(CoordError::new(
                        "DUPLICATE_OPTION",
                        format!("--{name} repeated"),
                    ));
                }
                options.flags.push(name.to_owned());
                index += 1;
                continue;
            }
            let value = args.get(index + 1).ok_or_else(|| {
                CoordError::new("MISSING_VALUE", format!("--{name} needs a value"))
            })?;
            options
                .values
                .entry(name.to_owned())
                .or_default()
                .push(value.clone());
            index += 2;
        }
        Ok(options)
    }

    pub(crate) fn one(&self, name: &str) -> Result<String, CoordError> {
        let values = self
            .values
            .get(name)
            .ok_or_else(|| CoordError::new("MISSING_OPTION", format!("--{name} is required")))?;
        if values.len() != 1 {
            return Err(CoordError::new(
                "DUPLICATE_OPTION",
                format!("--{name} must appear once"),
            ));
        }
        Ok(values[0].clone())
    }

    pub(crate) fn optional_one(&self, name: &str) -> Result<Option<String>, CoordError> {
        match self.values.get(name) {
            None => Ok(None),
            Some(values) if values.len() == 1 => Ok(Some(values[0].clone())),
            Some(_) => Err(CoordError::new(
                "DUPLICATE_OPTION",
                format!("--{name} must appear at most once"),
            )),
        }
    }

    pub(crate) fn many(&self, name: &str) -> Result<Vec<String>, CoordError> {
        self.values
            .get(name)
            .cloned()
            .ok_or_else(|| CoordError::new("MISSING_OPTION", format!("--{name} is required")))
    }

    pub(crate) fn u64_or(&self, name: &str, default: u64) -> Result<u64, CoordError> {
        let Some(value) = self.optional_one(name)? else {
            return Ok(default);
        };
        parse_ascii_u64(&value).ok_or_else(|| {
            CoordError::new("INVALID_OPTION", format!("--{name} has an invalid value"))
        })
    }

    pub(crate) fn u32_or(&self, name: &str, default: u32) -> Result<u32, CoordError> {
        let value = self.u64_or(name, u64::from(default))?;
        u32::try_from(value)
            .map_err(|_| CoordError::new("INVALID_OPTION", format!("--{name} exceeds u32")))
    }

    pub(crate) fn i32_or(&self, name: &str, default: i32) -> Result<i32, CoordError> {
        let Some(value) = self.optional_one(name)? else {
            return Ok(default);
        };
        parse_ascii_i32(&value).ok_or_else(|| {
            CoordError::new("INVALID_OPTION", format!("--{name} has an invalid value"))
        })
    }

    pub(crate) fn flag(&self, name: &str) -> bool {
        self.flags.iter().any(|flag| flag == name)
    }

    pub(crate) fn reject_flags(&self) -> Result<(), CoordError> {
        if self.flags.is_empty() {
            Ok(())
        } else {
            Err(CoordError::new(
                "UNKNOWN_OPTION",
                format!("unexpected --{}", self.flags[0]),
            ))
        }
    }

    pub(crate) fn reject_values(&self) -> Result<(), CoordError> {
        if self.values.is_empty() {
            Ok(())
        } else {
            let name = self.values.keys().next().expect("checked non-empty");
            Err(CoordError::new(
                "UNKNOWN_OPTION",
                format!("unexpected --{name}"),
            ))
        }
    }

    pub(crate) fn reject_unknown_flags(&self, allowed: &[&str]) -> Result<(), CoordError> {
        if let Some(flag) = self
            .flags
            .iter()
            .find(|flag| !allowed.contains(&flag.as_str()))
        {
            return Err(CoordError::new(
                "UNKNOWN_OPTION",
                format!("unexpected --{flag}"),
            ));
        }
        Ok(())
    }

    pub(crate) fn reject_unknown_values(&self, allowed: &[&str]) -> Result<(), CoordError> {
        if let Some(name) = self
            .values
            .keys()
            .find(|name| !allowed.contains(&name.as_str()))
        {
            return Err(CoordError::new(
                "UNKNOWN_OPTION",
                format!("unexpected --{name}"),
            ));
        }
        Ok(())
    }
}

pub(crate) fn parse_ascii_u64(value: &str) -> Option<u64> {
    let digits = value.strip_prefix('+').unwrap_or(value);
    if digits.is_empty() {
        return None;
    }
    digits.bytes().try_fold(0_u64, |number, byte| {
        byte.is_ascii_digit()
            .then_some(byte - b'0')
            .and_then(|digit| number.checked_mul(10)?.checked_add(u64::from(digit)))
    })
}

fn parse_ascii_i32(value: &str) -> Option<i32> {
    let (negative, digits) = if let Some(digits) = value.strip_prefix('-') {
        (true, digits)
    } else {
        (false, value.strip_prefix('+').unwrap_or(value))
    };
    let magnitude = parse_ascii_u64(digits)?;
    if negative {
        (magnitude <= i32::MAX as u64 + 1).then(|| -(magnitude as i64) as i32)
    } else {
        (magnitude <= i32::MAX as u64).then_some(magnitude as i32)
    }
}

#[cfg(test)]
mod tests {
    use super::Options;

    #[test]
    fn action_allowlists_reject_unused_options() {
        let options = Options::parse(&[
            "--agent".to_owned(),
            "agent-a".to_owned(),
            "--untrusted".to_owned(),
            "value".to_owned(),
        ])
        .unwrap();
        let error = options.reject_unknown_values(&["agent"]).unwrap_err();
        assert_eq!(error.code(), "UNKNOWN_OPTION");
    }
}
