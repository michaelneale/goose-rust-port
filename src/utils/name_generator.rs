use rand::{seq::SliceRandom, thread_rng};

/// Generate a random name in the format: letter-number-letter-number
/// Similar to how Star Wars droids are named (e.g. R2D2)
pub fn generate_name() -> String {
    let mut rng = thread_rng();
    let letters: Vec<char> = ('a'..='z').collect();
    let numbers: Vec<char> = ('0'..='9').collect();
    
    let letter1 = letters.choose(&mut rng).unwrap();
    let number1 = numbers.choose(&mut rng).unwrap();
    let letter2 = letters.choose(&mut rng).unwrap();
    let number2 = numbers.choose(&mut rng).unwrap();
    
    format!("{}{}{}{}", letter1, number1, letter2, number2)
}

#[cfg(test)]
mod tests {
    use super::*;
    use regex::Regex;

    #[test]
    fn test_generate_name() {
        let name = generate_name();
        
        // Name should be 4 characters long
        assert_eq!(name.len(), 4);
        
        // Name should match pattern letter-number-letter-number
        let pattern = Regex::new(r"^[a-z][0-9][a-z][0-9]$").unwrap();
        assert!(pattern.is_match(&name));
    }
}