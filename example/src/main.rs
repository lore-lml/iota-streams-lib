use std::fs::File;
use std::time::SystemTime;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use iota_streams_lib::channels::{ChannelWriter, ChannelReader};
use iota_streams_lib::payload::payload_serializers::{JsonPacketBuilder, JsonPacket};
use iota_streams_lib::utility::iota_utility::{create_encryption_key, create_encryption_nonce};

#[derive(Serialize, Deserialize, Debug)]
pub struct Message {
    pub device: String,
    pub operator: String,
    pub timestamp: u128,
    pub payload: MessagePayload
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MessagePayload {
    pub temperature: f32,
    pub humidity: f32,
    pub weight: f32
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct State {
    state: Vec<u8>,
}

fn current_time() -> Option<u128>{
    match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH){
        Ok(value) => Some(value.as_secs() as u128),
        Err(_) => None
    }
}


fn get_message(device_id: &str) -> Message{
    let fr = File::open("example/message.json").unwrap();
    let mut data: Message = serde_json::from_reader(fr).unwrap();
    data.timestamp = current_time().unwrap();
    data.device = device_id.to_string();
    data
}

async fn send_signed_message(channel: &mut ChannelWriter, device_id: &str, key: &[u8; 32], nonce: &[u8;24]){
    println!("Sending message ...");
    let p: Message = get_message(&format!("PUBLIC: {}", device_id));
    let m: Message = get_message(&format!("PRIVATE: {}", device_id));
    let data = JsonPacketBuilder::new()
        .public(&p).unwrap()
        .masked(&m).unwrap()
        .key_nonce(key, nonce)
        .build();
    let msg_id = channel.send_signed_packet(&data).await.unwrap();
    println!("... Message sent:");
    println!("  id: {}", msg_id);
    println!("  public: {:?}", p);
    println!("  masked: {:?}\n\n", m);
}

/*
USE https://chrysalis-nodes.iota.cafe/ for a node of the new chrysalis mainnet
*/
async fn test_channel_create(key: &[u8; 32], nonce: &[u8; 24], channel_psw: &str) -> Result<(String, String)>{
    let mut channel = ChannelWriter::builder().build();
    //let (channel_address, announce_id) = channels.open().await?;
    let (channel_address, announce_id, state_msg_id) = channel.open_and_save(channel_psw).await?;
    println!("Channel: {}:{}", &channel_address, &announce_id);
    println!("State Msg ID: {}", state_msg_id);
    println!("Announce Index: {}", channel.msg_index(&announce_id)?);

    for i in 1..=2{
        let device = format!("DEVICE_{}", i);
        send_signed_message(&mut channel, &device, key, nonce).await;
    }

    channel.export_to_file(channel_psw, "example/channels.state").await?;
    Ok((channel_address, announce_id))
}

async fn test_restore_channel(key: &[u8; 32], nonce: &[u8; 24], channel_psw: &str) -> Result<()>{
    println!("Restoring Channel ...");
    let mut channel = ChannelWriter::import_from_file(
        "example/channels.state",
        channel_psw,
        None,
        None
    ).await?;
    println!("... Channel Restored");

    let (channel_address, announce_id)= channel.channel_address();
    println!("Channel: {}:{}", &channel_address, announce_id);

    send_signed_message(&mut channel, "DEVICE_3", key, nonce).await;
    Ok(())
}

async fn test_restore_channel_from_tangle(channel: &str, announce: &str, key: &[u8; 32], nonce: &[u8; 24], state_psw: &str) -> Result<()>{
    println!("Restoring Channel from TANGLE...");
    let mut channel = ChannelWriter::import_from_tangle(
        channel,
        announce,
        state_psw,
        None,
        None).await?;
    println!("... Channel Restored from TANGLE");

    let (channel_address, announce_id)= channel.channel_address();
    println!("Channel: {}:{}", &channel_address, announce_id);

    send_signed_message(&mut channel, "DEVICE_4", key, nonce).await;
    Ok(())
}

async fn test_receive_messages(channel_id: &str, announce_id: &str, psw: &str, key: &[u8; 32], nonce: &[u8; 24]) -> Result<Vec<u8>>{
    let key_nonce = Some((key.clone(), nonce.clone()));

    let mut reader = ChannelReader::builder().build(channel_id, announce_id);
    reader.attach().await?;
    println!("Announce Received");

    print_msgs(&mut reader, key_nonce).await?;
    reader.export_to_bytes(psw).await
}

async fn test_restore_reader(state: &[u8], psw: &str, key: &[u8; 32], nonce: &[u8; 24]) -> Result<()>{
    let key_nonce = Some((key.clone(), nonce.clone()));
    println!("Restoring reader ...");
    let mut reader = ChannelReader::import_from_bytes(state, psw, None, None).await?;
    println!("... Reader restored");
    print_msgs(&mut reader, key_nonce).await
}

async fn print_msgs(reader: &mut ChannelReader, key_nonce: Option<([u8;32], [u8;24])>) -> Result<()>{
    let msgs = reader.fetch_parsed_msgs(&key_nonce).await.unwrap() as Vec<(String, JsonPacket)>;
    println!();
    for (id, packet) in msgs {
        println!("Message Found:");
        println!("  Msg Id: {}", id);
        let (p, m): (Message, Message) = packet.deserialize()?;
        println!("  Public: {:?}", p);
        println!("  Private: {:?}", m);
        println!("  Index: {}\n", reader.msg_index(&id)?);
    }
    println!("No more messages");
    Ok(())
}

#[tokio::main]
async fn main(){
    let key = create_encryption_key("This is a secret key");
    let nonce = create_encryption_nonce("This is a secret nonce");
    let channel_psw = "mypsw";
    let (channel, announce) = test_channel_create(&key, &nonce, channel_psw).await.unwrap();
    let state = test_receive_messages(&channel, &announce, channel_psw, &key, &nonce).await.unwrap();
    test_restore_channel(&key, &nonce, &channel_psw).await.unwrap();
    test_restore_channel_from_tangle(&channel, &announce, &key, &nonce, &channel_psw).await.unwrap();
    test_restore_reader(&state, channel_psw, &key, &nonce).await.unwrap();
}
