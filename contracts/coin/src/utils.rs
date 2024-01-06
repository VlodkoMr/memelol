use crate::*;

impl Contract {
    // Generate random u8 number (0-254)
    pub(crate) fn random_u8(&self, index: usize) -> u8 {
        *env::random_seed().get(index).unwrap()
    }

    // Get random number from 0 to max
    pub(crate) fn random_in_range(&self, index: usize, max: usize) -> u32 {
        if max > 0 {
            let rand_divider = 256f64 / (max + 1) as f64;
            let result = self.random_u8(index) as f64 / rand_divider;
            return result as u32;
        }
        0u32
    }

}
