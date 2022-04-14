use crate::PlayersRow;

const K_FACTOR_LIMIT: i32 = 20;
const K_FACTOR_STANDARD: f64 = 24.0;
const K_FACTOR_PROVISIONAL: f64 = 32.0;

pub struct Elo {
    pub player_one: PlayersRow,
    pub player_two: PlayersRow
}

impl Elo {
    fn expected_scores(&self, num_games: i32) -> (f64, f64) {
        let (mut ex_score_one, mut ex_score_two): (f64, f64) = (0.0, 0.0);

        for _games in 0..num_games {
            let quotient_one = f64::powf(10.0, self.player_one.player_elo / 400.0);
            let quotient_two = f64::powf(10.0, self.player_two.player_elo / 400.0);
            ex_score_one += quotient_one / (quotient_one + quotient_two);
            ex_score_two += quotient_two / (quotient_one + quotient_two);
        }

        return (ex_score_one, ex_score_two);
    }

    pub fn calculate_elo(&self, score_one: i32, score_two: i32) -> (f64, f64) {
        let k_factor_one = match self.player_one.player_num_games {
            i if i < K_FACTOR_LIMIT => K_FACTOR_PROVISIONAL,
            _ => K_FACTOR_STANDARD
        };
        let k_factor_two = match self.player_two.player_num_games {
            i if i < K_FACTOR_LIMIT => K_FACTOR_PROVISIONAL,
            _ => K_FACTOR_STANDARD
        };

        let (ex_score_one, ex_score_two) = self.expected_scores((score_one + score_two));
        let elo_one = self.player_one.player_elo + (k_factor_one * (score_one as f64 - ex_score_one));
        let elo_two = self.player_two.player_elo + (k_factor_two * (score_two as f64 - ex_score_two));
        return (elo_one, elo_two);
    }
}