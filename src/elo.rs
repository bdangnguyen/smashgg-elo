use crate::PlayersRow;

const K_FACTOR_LIMIT: i32 = 20;
const K_FACTOR_STANDARD: f64 = 24.0;
const K_FACTOR_PROVISIONAL: f64 = 32.0;

pub struct Elo {
    pub player_one: PlayersRow,
    pub player_one_score: i32,
    pub player_two: PlayersRow,
    pub player_two_score: i32
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

    pub fn calc_elo(&mut self) -> (f64, f64) {
        let k_factor_one = match self.player_one.player_num_games {
            i if i < K_FACTOR_LIMIT => K_FACTOR_PROVISIONAL,
            _ => K_FACTOR_STANDARD
        };
        let k_factor_two = match self.player_two.player_num_games {
            i if i < K_FACTOR_LIMIT => K_FACTOR_PROVISIONAL,
            _ => K_FACTOR_STANDARD
        };

        let (ex_score_one, ex_score_two) = self.expected_scores(self.player_one_score + self.player_two_score);
        let mut player_one_delta = k_factor_one * (self.player_one_score as f64 - ex_score_one);
        let mut player_two_delta = k_factor_two * (self.player_two_score as f64 - ex_score_two);

        if self.player_one_score == -1 || self.player_two_score == -1 {
            player_one_delta = 0.0;
            player_two_delta = 0.0;
        }

        self.player_one.player_elo += player_one_delta;
        self.player_one.player_num_games += self.player_one_score + self.player_two_score;
        self.player_one.player_wins += self.player_one_score;
        self.player_one.player_losses += self.player_two_score;
        self.player_one.player_win_loss_ratio = self.player_one.player_wins as f64 / self.player_one.player_num_games as f64;

        self.player_two.player_elo += k_factor_two * (self.player_two_score as f64 - ex_score_two);
        self.player_two.player_num_games += self.player_one_score + self.player_two_score;
        self.player_two.player_wins += self.player_two_score;
        self.player_two.player_losses += self.player_one_score;
        self.player_two.player_win_loss_ratio = self.player_two.player_wins as f64 / self.player_two.player_num_games as f64;

        return (player_one_delta, player_two_delta);
    }
}