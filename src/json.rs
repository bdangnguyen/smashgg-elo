use crate::reqwest_wrapper::{Content, ContentType, ReqwestClient};
use std::collections::HashMap;
use serde::Deserialize;
use smashgg_elo_rust::get_input;

const EVNT_PROMPT: &str = "Enter the id of one of the events to parse: ";

/// Generic entry point for all post requests. The top level struct acts as
/// the object that the rest of the program interfaces with to get the data.
#[derive(Deserialize, Debug)] 
pub struct PostResponse { 
    data: Data
}

impl PostResponse {
    /// Once the initial post request has been made, this function takes the
    /// JSON response and parses it for all events in a tournament. The user
    /// will then be continually asked for which event they wish to parse.
    pub fn get_event_info(self) -> (i32, String, String) {
        let tournament = self.data.tournament();
        let num_evnts: i32 = (tournament.events.len() - 1).try_into().unwrap();
        
        // Print all events to the console with the associated game and event
        // name. Loop continuously until the user selects one to parse.
        loop {

            println!("List of events found in the tournament:");
            for (count, event) in tournament.events.iter().enumerate() {
                println!("{}: {:?} - {:?}", count, event.videogame.name, event.name);
            }

            let event_input: i32 = get_input(EVNT_PROMPT);
            match event_input {
                i if i < 0 => continue,
                i if i > num_evnts => continue,
                _ =>  {
                    let info = &tournament.events[event_input as usize];
                    return (
                        info.id,
                        info.videogame.name.to_owned(),
                        info.name.to_owned()
                    );
                }
            };
        }
    }

    pub fn get_total_pages(self) -> i32 {
        self.data.event().sets().page_info().total_pages
    }

    /// Gets all sets in an event. Iterates through the all of the sets and
    /// their results in an event and records them in a vector that will be
    /// later parsed.
    pub fn get_sets_info(self) -> Vec<SetInfo> {
        let mut set_vec = Vec::new();

        let player_nodes = self.data.event().sets().nodes();
        for node in player_nodes {
            let player_one = &node.slots()[0];
            let player_two = &node.slots()[1];

            set_vec.push(
                SetInfo {
                    player_one_id: player_one.entrant().id(),
                    player_one_score: player_one.standing().stats.score.value(),
                    player_two_id: player_two.entrant().id(),
                    player_two_score: player_two.standing().stats.score.value(),
                    time: node.completed_at()
                }
            );
        }
        set_vec.reverse();
        set_vec
    }

    /// Repeatedly queries smash.gg's api and collects all of the players
    /// that participated in an event. Maps each player's tournament id to
    /// their global smash.gg id and name.
    pub fn construct_players(
        self,
        reqwest_client: &mut ReqwestClient,
        event_id: i32
    ) -> HashMap<i32, (String, i32)> {
        // Detect how many calls to the api that we need to make to record
        // all players in an event.
        let mut player_map = HashMap::new();
        let page_info = self.data.event().entrants().page_info();
        println!("Constructing the list of players...");

        // Call multiple times and record each player that participated.
        println!("Found {} pages of player data", page_info.total_pages);
        for i in 1..page_info.total_pages + 1 {
            println!("Processing page {} out of {}...",i, page_info.total_pages);
            let mut content = Content::new();
            content.variables.event_id = Some(event_id);
            content.variables.page = Some(i);
            content.edit_content(ContentType::Page);
            reqwest_client.construct_json(&content);

            let json: PostResponse = match reqwest_client.send_post().json() {
                Ok(json) => json,
                Err(err) => panic!("Error in converting to json {}", err),
            };

            let nodes = json.data.event().entrants().nodes();

            for player in nodes {
                player_map.insert(
                 player.id(),
                 (player.participants()[0].gamer_tag.to_owned(),
                    player.participants()[0].user.id()),
                );
            }
        }

        player_map
    }

}
#[derive(Deserialize, Debug)] 
struct Data { 
    tournament: Option<Tournament>,
    event: Option<Event>
}

impl Data {
    fn tournament(&self) -> &Tournament {
        self.tournament.as_ref().expect("Matching error: No tournament found")

    }
    
    fn event(self) -> Event {
        self.event.expect("Matching error: No event found")
    }
}

#[derive(Deserialize, Debug)] 
struct Tournament { 
    events: Vec<Events>, 
}

#[derive(Deserialize, Debug)] 
struct Events {
    id: i32,
    name: String,
    videogame: Videogame
}
#[derive(Deserialize, Debug)]
struct Videogame {
    name: String
}
#[derive(Deserialize, Debug)] 
struct Event {
    entrants: Option<Entrants>,
    sets: Option<Sets>
}

impl Event {
    fn entrants(self) -> Entrants {
        self.entrants.expect("Matching error: No entrants found")
    }

    fn sets(self) -> Sets {
        self.sets.expect("Matching error: No sets found")
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
 struct Entrants {
    page_info: Option<PageInfo>,
    nodes: Option<Vec<Nodes>>
}

impl Entrants {
    fn page_info(self) -> PageInfo {
        self.page_info.expect("Matching error: No page info found in entrants")
    }

    fn nodes(self) -> Vec<Nodes> {
        self.nodes.expect("Matching error: No nodes found in entrants")
    }
}


#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Sets {
    page_info: Option<PageInfo>,
    nodes: Option<Vec<Nodes>>
}

impl Sets {
    fn page_info(self) -> PageInfo {
        self.page_info.expect("Matching error: No page info found in sets")
    }

    fn nodes(self) -> Vec<Nodes> {
        self.nodes.expect("Matching error: No nodes found in sets")
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
 struct PageInfo {
    total_pages: i32
}
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
 struct Nodes {
    id: Option<i32>,
    participants: Option<Vec<Participants>>,
    completed_at: Option<i64>,
    slots: Option<Vec<Slots>>,
}

impl Nodes {
    fn id(&self) -> i32 {
        self.id.expect("Matching error: No id found")
    }

    fn participants(&self) -> &Vec<Participants> {
        self.participants.as_ref().expect("Matching error: No participants found")
    }

    fn completed_at(&self) -> i64 {
        self.completed_at.expect("Matching error: No time found")
    }

    fn slots(&self) -> &Vec<Slots> {
        self.slots.as_ref().expect("Matching error: No slots found")
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Participants {
    gamer_tag: String,
    user: User
}
#[derive(Deserialize, Debug)]
struct User {
    id: Option<i32>
}

impl User {
    fn id(&self) -> i32 {
        self.id.expect("Matching error in user: No id found")
    }
}

#[derive(Deserialize, Debug)]
struct Slots {
    entrant: Option<Entrant>,
    standing: Option<Standing>
}

impl Slots {
    fn entrant(&self) -> &Entrant {
        self.entrant.as_ref().expect("Matching error: No entrant found")
    }

    fn standing(&self) -> &Standing {
        self.standing.as_ref().expect("Matching error: No standing found")
    }
}
#[derive(Deserialize, Debug)]
struct Entrant {
    id: Option<i32>
}

impl Entrant {
    fn id(&self) -> i32 {
        self.id.expect("Matching error in entrant: No id found")
    }
}

#[derive(Deserialize, Debug)]
struct Standing {
    stats: Stats
}
#[derive(Deserialize, Debug)]
struct Stats {
    score: Score
}
#[derive(Deserialize, Debug)]
struct Score {
    value: Option<i32>
}

// This handles the edge case where we request data and find out that there
// is none for a set. We thus then treat it as iff they have been DQ'd.
impl Score {
    fn value(&self) -> i32 {
        match self.value {
            Some(value) => value,
            None => -1,
        }
    }
}

/// Internal struct used to contain information about the results of a set.
pub struct SetInfo {
    pub player_one_id: i32,
    pub player_one_score: i32,
    pub player_two_id: i32,
    pub player_two_score: i32,
    pub time: i64
}