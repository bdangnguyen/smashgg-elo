use std::collections::HashMap;
use serde::Deserialize;
use smashgg_elo_rust::get_input;

use crate::reqwest_wrapper::Content;
use crate::reqwest_wrapper::ReqwestClient;
use crate::reqwest_wrapper::ContentType;

const SLUG_PROMPT: &str = "Enter the tournament slug to read data from: ";
const EVNT_PROMPT: &str = "Enter the id of one of the events to parse: ";
const MAX_ENTRANTS: i32 = 499;
const MAX_SETS: i32 = 70;

#[derive(Deserialize, Debug)] 
pub struct PostResponse { 
    data: Data
}

impl PostResponse {
    pub fn get_event_info(self) -> (i32, String) {
        let tournament = self.data.tournament();
        let num_evnts: i32 = (tournament.events.len() - 1).try_into().unwrap();
        
        loop {
            let mut count = 0;

            println!("List of events found in the tournament:");
            for event in &tournament.events {
                println!("{}: {:?}", count, event.name);
                count += 1;
            }

            let event_input: i32 = get_input(EVNT_PROMPT);
            match event_input {
                i if i < 0 => continue,
                i if i > (num_evnts).try_into().unwrap() => continue,
                _ =>  {
                    let info = &tournament.events[event_input as usize];
                    return (info.id, info.name.to_owned());
                }
            };
        }
    }

    pub fn get_total_pages(self) -> i32 {
        self.data.event().sets().page_info().total_pages
    }

    pub fn get_sets_info(self) -> Vec<(i32, i32, i32, i32, i64)> {
        let mut set_info = Vec::new();

        let player_nodes = self.data.event().sets().nodes();
        for node in player_nodes {
            let player_one = &node.slots()[0];
            let player_two = &node.slots()[1];
            let player_one_id = player_one.entrant.id;
            let player_two_id = player_two.entrant.id;
            let player_one_score = player_one.standing.stats.score.value;
            let player_two_score = player_two.standing.stats.score.value;
            set_info.push((player_one_id, player_one_score, player_two_id, player_two_score, node.completed_at()));
        }

        return set_info;
    }

    pub fn construct_player_map(self, reqwest_client: &mut ReqwestClient, event_id: i32) -> HashMap<i32, (String, i32)>{
        let mut player_map = HashMap::new();

        let page_info = self.data.event().entrants().page_info();

        for i in 0.. page_info.total_pages {
            let mut content = Content::new();
            content.variables.event_id = Some(event_id);
            content.variables.page = Some(i);
            content.edit_content(ContentType::PageContent);
            reqwest_client.construct_json(&content);

            let result = reqwest_client.send_post();

            let json: PostResponse = match result.json() {
                Ok(json) => json,
                Err(err) => panic!("Error in converting to json {}", err),
            };

            let nodes = json.data.event().entrants().nodes();

            for player in nodes {
                player_map.insert(
                    player.id(),
                    (player.participants()[0].gamer_tag.to_owned(), player.participants()[0].user.id),
                );
            }
        }

        //println!("Map: {:?}", player_map);

        return player_map;
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
        self.nodes.expect("Matching error: No nodes found in etrants")
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
    id: i32
}

#[derive(Deserialize, Debug)]
struct Slots {
    entrant: Entrant,
    standing: Standing
}
#[derive(Deserialize, Debug)]
struct Entrant {
    id: i32
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
    value: i32
}