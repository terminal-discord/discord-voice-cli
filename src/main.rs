extern crate clap;
extern crate cpal;
extern crate serenity;

use std::io::stdin;
use std::sync::{mpsc, Arc};
use std::thread;

use serenity::{
    client::{Client, Context, EventHandler},
    model::prelude::*,
    prelude::Mutex,
};

mod args;
mod receiver;
mod sender;

struct Handler(pub Arc<Mutex<mpsc::Sender<()>>>);

impl EventHandler for Handler {
    fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
        self.0.lock().send(()).unwrap();
    }
}

fn main() {
    let args = args::get_args();
    let token = args.token.expect("token");
    let guild_id = args.guild_id.expect("guild");
    let channel_id = args.channel_id.expect("channel");
    // let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    // let guild_id = GuildId(
    //     env::var("DISCORD_GUILD")
    //         .expect("Expected a token in the environment")
    //         .parse::<u64>()
    //         .expect("Invalid int"),
    // );
    // let chan_id = ChannelId(
    //     env::var("DISCORD_CHANNEL")
    //         .expect("Expected a token in the environment")
    //         .parse::<u64>()
    //         .expect("Invalid int"),
    // );

    let (ready_tx, ready_rx) = mpsc::channel();
    let (close_tx, close_rx) = mpsc::channel();
    let (closed_tx, closed_rx) = mpsc::channel();

    let mut client =
        Client::new(&token, Handler(Arc::new(Mutex::new(ready_tx)))).expect("Err creating client");

    let voice_man = Arc::clone(&client.voice_manager);
    let shutdown_voice_manager = Arc::clone(&client.voice_manager);

    thread::spawn(move || {
        client.start().unwrap();
    });

    thread::spawn(move || {
        close_rx.recv().unwrap();
        let mut voice_man_lock = shutdown_voice_manager.lock();
        if let Some(handler) = voice_man_lock.get_mut(guild_id) {
            handler.leave();
            closed_tx.send(()).unwrap();
        }
    });

    thread::spawn(move || {
        ready_rx.recv().unwrap();
        let mut voice_man_lock = voice_man.lock();
        voice_man_lock.join(guild_id, channel_id).unwrap();
        if let Some(handler) = voice_man_lock.get_mut(guild_id) {
            let rec = receiver::Receiver::new();
            handler.listen(Some(Box::new(rec)));

            let sender = sender::Sender::new();
            handler.play(Box::new(sender));
        }
    });

    let mut buf = String::new();
    stdin().read_line(&mut buf);

    // thread::sleep_ms(100_000);
    close_tx.send(()).unwrap();
    closed_rx.recv().unwrap();
    println!("Disconnected");
}
