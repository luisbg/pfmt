use std::collections::HashMap;
use std::iter::repeat;
use std::string::ToString;

use num;

use {SingleFmtError, SingleFmtError::*};

/* ---------- general formatting options ---------- */

pub enum Justification {
    Left(),
    Center(),
    Right()
}

pub fn apply_common_options(s: &mut String, options: &HashMap<String, String>)
    -> Result<(), SingleFmtError>
{
    apply_truncation(s, options)?;
    apply_width(s, options)?;
    Ok(())
}

pub fn apply_width(s: &mut String, options: &HashMap<String, String>)
    -> Result<(), SingleFmtError>
{
    if let Some(width_str) = options.get("width") {
        let justification = match width_str.chars().nth(0) {
            Some('l') => Justification::Left(),
            Some('c') => Justification::Center(),
            Some('r') => Justification::Right(),
            _ => return Err(SingleFmtError::InvalidOptionValue("width".to_string(), 
                width_str.to_string()))
        };
        if let Ok(width) = width_str[1..].parse::<usize>() {
            let len = s.chars().count();
            if len > width {
                return Ok(());
            }
            let delta = width - len;
            match justification {
                Justification::Left() => {
                    let padding: String = repeat(' ').take(delta).collect();
                    s.push_str(&padding);
                }
                Justification::Center() => {
                    let left = delta / 2;
                    let right = delta - left;
                    let leftstr: String = repeat(' ').take(left).collect();
                    let rightstr: String = repeat(' ').take(right).collect();
                    *s = leftstr + s + &rightstr;
                }
                Justification::Right() => {
                    let padding: String = repeat(' ').take(delta).collect();
                    *s = padding + s;
                }
            }
        } else {
            return Err(InvalidOptionValue("width".to_string(),
                width_str.to_string()));
        }
    }
    Ok(())
}

pub fn apply_truncation(s: &mut String, options: &HashMap<String, String>)
    -> Result<(), SingleFmtError>
{
    if let Some(opt_str) = options.get("truncate") {
        if opt_str.is_empty() {
            return Err(InvalidOptionValue("truncate".to_string(), opt_str.to_string()));
        }
        let is_left = match opt_str.chars().nth(0) {
            Some('l') => true,
            Some('r') => false,
            _ => return Err(InvalidOptionValue("truncate".to_string(), opt_str.to_string()))
        };
        if let Ok(truncate_to_width) = opt_str[1..].parse::<usize>() {
            let len = s.chars().count();
            if len < truncate_to_width {
                return Ok(());
            }
            if is_left {
                *s = s.chars().skip(len - truncate_to_width).collect();
            } else {
                *s = s.chars().take(truncate_to_width).collect();
            }
            Ok(())
        } else {
            Err(InvalidOptionValue("truncate".to_string(), opt_str.to_string()))
        }
    } else {
        Ok(())
    }
}

/* ---------- numerical formatting ---------- */

pub fn float_to_string<T>(f: T, exp: bool, options: &HashMap<String, String>)
    -> Result<String, SingleFmtError>
    where T: num::Float + num::FromPrimitive + ToString
{
    let precision = {
        if let Some(s) = options.get("prec") {
            match s.parse::<i32>() {
                Ok(i) => Some(i),
                Err(_) => return Err(SingleFmtError::InvalidOptionValue(
                    "prec".to_string(),
                    s.to_string()))
            }
        } else {
            None
        }
    };
    if exp {
        match precision {
            Some(p) if p <= 0 => {
            },
            Some(_p) => {
                Ok("".to_string())
            },
            None => {
                Ok("".to_string())
            }
        }
    } else {
        Ok("".to_string())
    }
}

pub fn apply_common_numeric_options(s: &mut String, 
                                    flags: &[char],
                                    _options: &HashMap<String, String>)
    -> Result<(), SingleFmtError>
{
    if s.is_empty() {
        return Ok(());
    }
    if flags.contains(&'+') && s.chars().nth(0).unwrap() != '-' {
        let mut new_s = String::with_capacity(1 + s.len());
        new_s.push('+');
        new_s += s;
        *s = new_s;
    }
    Ok(())
}
