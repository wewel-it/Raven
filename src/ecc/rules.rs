use crate::ecc::traits::Rule;

/// Sekumpulan rule yang dapat digunakan pada validator.
pub struct RuleSet<T> {
    pub rules: Vec<Box<dyn Rule<T>>>,
}

impl<T> RuleSet<T> {
    /// Buat RuleSet baru.
    pub fn new(rules: Vec<Box<dyn Rule<T>>>) -> Self {
        Self { rules }
    }

    /// Tambahkan rule baru ke dalam set.
    pub fn add_rule(&mut self, rule: Box<dyn Rule<T>>) {
        self.rules.push(rule);
    }
}
