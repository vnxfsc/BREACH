//! Gene calculation utilities

/// Gene rank classification
pub fn gene_rank(value: u8) -> char {
    match value {
        251..=255 => 'S',
        201..=250 => 'A',
        151..=200 => 'B',
        101..=150 => 'C',
        51..=100 => 'D',
        _ => 'F',
    }
}

/// Calculate overall gene quality score (0-1530)
pub fn gene_score(genes: &[u8; 6]) -> u16 {
    genes.iter().map(|g| *g as u16).sum()
}

/// Get gene grade string
pub fn gene_grade(genes: &[u8; 6]) -> &'static str {
    match gene_score(genes) {
        1401..=1530 => "SSS",
        1201..=1400 => "SS",
        1001..=1200 => "S",
        801..=1000 => "A",
        601..=800 => "B",
        401..=600 => "C",
        201..=400 => "D",
        _ => "F",
    }
}

/// Calculate offspring genes from two parents
/// 
/// Gene inheritance rules:
/// - 45% chance: inherit from parent A
/// - 45% chance: inherit from parent B  
/// - 10% chance: mutation (average ± random)
pub fn calculate_offspring_genes(
    parent_a: &[u8; 6],
    parent_b: &[u8; 6],
    randomness: &[u8; 32],
) -> [u8; 6] {
    let mut offspring = [0u8; 6];

    for i in 0..6 {
        let roll = randomness[i] % 100;

        if roll < 45 {
            // 45% from parent A
            offspring[i] = parent_a[i];
        } else if roll < 90 {
            // 45% from parent B
            offspring[i] = parent_b[i];
        } else {
            // 10% mutation
            let avg = ((parent_a[i] as i16 + parent_b[i] as i16) / 2) as i16;
            let mutation = (randomness[i + 6] as i16 % 64) - 32; // -32 to +31
            offspring[i] = (avg + mutation).clamp(0, 255) as u8;
        }
    }

    offspring
}

/// Calculate base damage in combat
pub fn calculate_damage(
    attacker_power: u8,
    defender_fortitude: u8,
    skill_power: u8,
    type_multiplier: u8, // 75, 100, or 150
    randomness: u8,
) -> u16 {
    let attack = attacker_power as u32;
    let defense = (defender_fortitude as u32).max(1);
    let skill = skill_power as u32;
    let type_mult = type_multiplier as u32;

    // Random factor: 85-100%
    let random_factor = 85 + (randomness as u32 % 16);

    let damage = (attack * skill / defense) * type_mult / 100 * random_factor / 100;

    damage.max(1).min(65535) as u16
}

/// Calculate Elo rating change after a match
pub fn calculate_elo_change(winner_rating: u16, loser_rating: u16) -> (i16, i16) {
    const K: i32 = 32;

    let expected_winner =
        1.0 / (1.0 + 10.0_f64.powf((loser_rating as i32 - winner_rating as i32) as f64 / 400.0));
    let expected_loser = 1.0 - expected_winner;

    let delta_winner = (K as f64 * (1.0 - expected_winner)) as i16;
    let delta_loser = (K as f64 * (0.0 - expected_loser)) as i16;

    (delta_winner, delta_loser)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gene_rank() {
        assert_eq!(gene_rank(255), 'S');
        assert_eq!(gene_rank(251), 'S');
        assert_eq!(gene_rank(250), 'A');
        assert_eq!(gene_rank(201), 'A');
        assert_eq!(gene_rank(200), 'B');
        assert_eq!(gene_rank(151), 'B');
        assert_eq!(gene_rank(150), 'C');
        assert_eq!(gene_rank(101), 'C');
        assert_eq!(gene_rank(100), 'D');
        assert_eq!(gene_rank(51), 'D');
        assert_eq!(gene_rank(50), 'F');
        assert_eq!(gene_rank(0), 'F');
    }

    #[test]
    fn test_gene_score() {
        assert_eq!(gene_score(&[255, 255, 255, 255, 255, 255]), 1530);
        assert_eq!(gene_score(&[0, 0, 0, 0, 0, 0]), 0);
        assert_eq!(gene_score(&[100, 100, 100, 100, 100, 100]), 600);
    }

    #[test]
    fn test_gene_grade() {
        assert_eq!(gene_grade(&[255, 255, 255, 255, 255, 255]), "SSS");
        assert_eq!(gene_grade(&[200, 200, 200, 200, 200, 200]), "S");
        assert_eq!(gene_grade(&[100, 100, 100, 100, 100, 100]), "C");
    }

    #[test]
    fn test_offspring_genes() {
        let parent_a = [200, 150, 180, 220, 190, 210];
        let parent_b = [180, 200, 160, 190, 210, 180];
        let randomness = [50u8; 32];

        let offspring = calculate_offspring_genes(&parent_a, &parent_b, &randomness);

        // All genes should be in valid range
        for gene in offspring.iter() {
            assert!(*gene <= 255);
        }
    }

    #[test]
    fn test_elo_change() {
        let (winner_delta, loser_delta) = calculate_elo_change(1000, 1000);
        assert_eq!(winner_delta, 16);
        assert_eq!(loser_delta, -16);

        let (winner_delta, loser_delta) = calculate_elo_change(1200, 1000);
        assert!(winner_delta < 16); // Lower gain for expected win
        assert!(loser_delta > -16); // Lower loss for expected loss
    }
}
