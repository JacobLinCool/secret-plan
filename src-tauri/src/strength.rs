use crate::traits::PasswordStrengthCalculator;
use zxcvbn::zxcvbn;

/// Simple password strength calculator.
pub struct SimpleStrengthCalculator;

impl PasswordStrengthCalculator for SimpleStrengthCalculator {
    fn calculate_strength(&self, password: &str) -> u8 {
        // Use zxcvbn to calculate password strength
        let estimate = zxcvbn(password, &[]);

        // zxcvbn returns a score from 0 to 4, score less than 3 should be considered too weak
        estimate.score() as u8
    }
}
