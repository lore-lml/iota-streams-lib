use iota_streams::app::transport::{
    TransportOptions,
    tangle::{
        client::{Client as StreamsClient, SendOptions},
        PAYLOAD_BYTES
    }
};
use iota_streams::app_channels::api::tangle::Author;
use crate::utility::iota_utility::random_seed;
use anyhow::Result;

pub struct AuthorBuilder{
    seed: String,
    node_url: String,
    encoding: String,
    multi_branching: bool,
    send_options: SendOptions
}

impl AuthorBuilder{
    pub fn new() -> AuthorBuilder{
        AuthorBuilder{
            seed: random_seed(),
            node_url: "https://api.lb-0.testnet.chrysalis2.com".to_string(),
            encoding: "utf-8".to_string(),
            multi_branching: false,
            send_options: SendOptions::default()
        }
    }
}

impl AuthorBuilder{
    pub fn seed(mut self, seed: &str) -> Self{
        self.seed = seed.to_string();
        self
    }

    pub fn node(mut self, node_url: &str) -> Self{
        self.node_url = node_url.to_string();
        self
    }

    pub fn encoding(mut self, encoding: &str) -> Self{
        self.encoding = encoding.to_string();
        self
    }

    pub fn send_options(mut self, send_options: SendOptions) -> Self{
        self.send_options = send_options;
        self
    }

    pub fn build(mut self) -> Result<Author<StreamsClient>>{
        /*if self.seed != "" && !self.seed_ok(){
            return None;
        }else*/

        let mut client = StreamsClient::new_from_url(&self.node_url);
        client.set_send_options(self.send_options);
        let author = Author::new(&self.seed,
                                 &self.encoding,
                                 PAYLOAD_BYTES,
                                 self.multi_branching,
                                 client);
        Ok(author)
    }

    /*fn seed_ok(self) -> bool{
        /*if self.seed != "" && self.seed.len() != 81 {
            return false;
        }*/

        let char_set: Vec<char> = "9ABCDEFGHIJKLMNOPQRSTUVWXYZ".chars().collect();
        let char_set: HashSet<char> = HashSet::from_iter(char_set.iter().cloned());

        let found_char_set : Vec<char> = self.seed.clone().chars().collect();
        let found_char_set : HashSet<char> = HashSet::from_iter(found_char_set.iter().cloned());

        found_char_set.is_subset(&char_set)
    }*/
}
