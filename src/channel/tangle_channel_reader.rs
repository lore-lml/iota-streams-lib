use anyhow::Result;
use iota_streams::{
    app::transport::tangle::client::{Client as StreamsClient},
    app_channels::api::tangle::{Address, Subscriber},
};

use std::string::ToString;
use crate::payload::payload_serializer::json::Payload;
use serde::de::DeserializeOwned;
use iota_streams::app_channels::api::tangle::MessageContent;

///
/// Channel
///
pub struct ChannelReader {
    subscriber: Subscriber<StreamsClient>,
    channel_address: String,
    announcement_id: String,
    //next_msg_id: String,
}

impl ChannelReader {
    ///
    /// Initialize the Channel Reader
    ///
    pub fn new(subscriber: Subscriber<StreamsClient>, channel_address: &str, announcement_id: &str) -> ChannelReader {
        ChannelReader {
            subscriber,
            channel_address: channel_address.to_string(),
            announcement_id: announcement_id.to_string(),
            //next_msg_id: String::default(),
        }
    }

    ///
    /// Open a Channel Reader
    ///
    pub fn open(&mut self) -> Result<()> {
        let link = Address::from_str(&self.channel_address, &self.announcement_id).unwrap();
        self.subscriber.receive_announcement(&link)
    }

    ///
    /// Receive signed packet
    ///
    pub fn receive_signed<'a, T>(&mut self, msg_id: &str) -> Result<T>
        where
            T: DeserializeOwned
    {
        let msg_link = Address::from_str(&self.channel_address, msg_id).unwrap();
        let (_, public_payload, _) = self.subscriber.receive_signed_packet(&msg_link)?;
        let p_data: T = Payload::unwrap_data(public_payload.0).unwrap();
        Ok(p_data)
    }

    pub fn fetch_remaining_msgs(&mut self) -> Vec<(String, Vec<u8>)> {
        let mut result: Vec<(String, Vec<u8>)> = Vec::new();

        let mut has_next = true;

        while has_next{
            let mut msgs: Vec<(String, Vec<u8>)> = self.subscriber.fetch_next_msgs().iter()
                .map(|msg| {
                    let link = msg.link.to_string().clone();

                    let (p_data, _) = match &msg.body {
                        MessageContent::SignedPacket {public_payload, masked_payload, .. } => (public_payload.0.clone(), masked_payload.0.clone()),
                        _ => (Vec::new(), Vec::new())
                    };

                    (link, p_data)
                })
                .filter(|msg|  {
                    msg.1.len() > 0
                })
                .collect();

            if msgs.len() > 0{
                result.append(&mut msgs);
            }else {
                has_next = false;
            }
        }

        result
    }

    ///
    /// Get the channel address
    ///
    pub fn channel_address(&self) -> (String, String){
        (self.channel_address.clone(), self.announcement_id.clone())
    }
}