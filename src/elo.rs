use crate::PlayersRow;

const K_FACTOR_LIMIT: i32 = 20;
const K_FACTOR_STANDARD: f64 = 24.0;
const K_FACTOR_PROVISIONAL: f64 = 32.0;

pub struct Elo {
    pub player_one: PlayersRow,
    pub score_one: i32,
    pub player_two: PlayersRow,
    pub score_two: i32
}

impl Elo {
    /// Calculates the expected score used for the Elo algorithm. The formula
    /// for a player's expected score is given as
    /// 
    /// E_{p_1} = Q_{p_1} / (Q_{p_1} + Q_{p_2})
    /// 
    /// Where Q_{p_1} = 10^(Elo_{p_1} / 400), Q_{p_2} = 10^(Elo_{p_2} / 400)
    fn expected_scores(&self, num_games: i32) -> (f64, f64) {
        let (mut ex_score_one, mut ex_score_two) = (0.0, 0.0);

        for _games in 0..num_games {
            let quotient_one = f64::powf(10.0, self.player_one.elo / 400.0);
            let quotient_two = f64::powf(10.0, self.player_two.elo / 400.0);
            ex_score_one += quotient_one / (quotient_one + quotient_two);
            ex_score_two += quotient_two / (quotient_one + quotient_two);
        }

        return (ex_score_one, ex_score_two);
    }

    /// Calculates the actual elo changes given two players and their respective
    /// scores in a set. This will only be called when both player's finish
    /// a set completely.
    pub fn calc_elo(&mut self) -> (f64, f64) {
        // Calculate the k-factor for each player. If the player has played
        // under 20 total games for any game, they are assigned provisional.
        let k_factor_one = match self.player_one.num_games {
            i if i < K_FACTOR_LIMIT => K_FACTOR_PROVISIONAL,
            _ => K_FACTOR_STANDARD
        };
        let k_factor_two = match self.player_two.num_games {
            i if i < K_FACTOR_LIMIT => K_FACTOR_PROVISIONAL,
            _ => K_FACTOR_STANDARD
        };

        // Calculate the change in elo for both players. The formula for a
        // player's change in elo is given as
        //
        // delta = k_factor * (score - ex_score)
        let num_games = self.score_one + self.score_two;
        let (ex_score_one, ex_score_two) = self.expected_scores(num_games);
        let delta_one = k_factor_one * (self.score_one as f64 - ex_score_one);
        let delta_two = k_factor_two * (self.score_two as f64 - ex_score_two);

        self.player_one.elo += delta_one;
        self.player_one.num_games += self.score_one + self.score_two;
        self.player_one.wins += self.score_one;
        self.player_one.losses += self.score_two;
        self.player_one.win_loss_ratio = self.player_one.wins as f64 / self.player_one.num_games as f64;

        self.player_two.elo += delta_two;
        self.player_two.num_games += self.score_one + self.score_two;
        self.player_two.wins += self.score_two;
        self.player_two.losses += self.score_one;
        self.player_two.win_loss_ratio = self.player_two.wins as f64 / self.player_two.num_games as f64;

        return (delta_one, delta_two);
    }
}