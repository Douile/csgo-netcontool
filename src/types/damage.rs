#[derive(Debug, PartialEq)]
pub enum DamageDirection {
    Given,
    Taken,
}

#[derive(Debug, PartialEq)]
pub struct Damage {
    pub direction: DamageDirection,
    pub target: String,
    pub amount: u8,
    pub hits: u8,
}

impl TryFrom<&str> for Damage {
    type Error = &'static str;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut value = value
            .strip_prefix("Damage ")
            .ok_or("Invalid damage string 1")?;

        let direction = if value.starts_with("Taken from") {
            value = &value[10..];
            DamageDirection::Taken
        } else if value.starts_with("Given to") {
            value = &value[8..];
            DamageDirection::Given
        } else {
            Err("Unknown damage direction")?
        };

        let value = value.strip_prefix(" \"").ok_or("Invalid damage string 3")?;

        let (target, value) = value.split_once("\"").ok_or("Invalid damage string 4")?;

        let value = value.strip_prefix(" - ").ok_or("Invalid damage string 5")?;

        let (amount, value) = value.split_once(" ").ok_or("Invalid damage string 6")?;

        let value = value.strip_prefix("in ").ok_or("Invalid damage string 7")?;

        let (hits, _) = value.split_once(" ").ok_or("Invalid damage string 8")?;

        Ok(Damage {
            direction,
            target: target.to_string(),
            amount: u8::from_str_radix(amount, 10).or(Err("Invalid damage amount"))?,
            hits: u8::from_str_radix(hits, 10).or(Err("Invalid damage hits amount"))?,
        })
    }
}
